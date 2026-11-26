#!/usr/bin/env python3
"""Salva em texto as legendas disponíveis de um vídeo do YouTube.

Instalação: python3 -m pip install 'youtube-transcript-api>=1.0,<2'
Exemplo: python3 transcrever_youtube.py 'https://youtu.be/bQCBfcZZj0Y'
"""

import argparse
import re
import sys
from pathlib import Path
from urllib.parse import parse_qs, urlparse

VIDEO_PADRAO = 'https://youtu.be/bQCBfcZZj0Y'


def extrair_id(entrada):
    """Aceita ID ou links watch, youtu.be, shorts, embed e live."""
    if re.fullmatch(r'[\w-]{11}', entrada):
        return entrada
    url = urlparse(entrada)
    host = (url.hostname or '').lower()
    partes = url.path.strip('/').split('/')
    video_id = None
    if host in ('youtu.be', 'www.youtu.be'):
        video_id = partes[0]
    elif host in ('youtube.com', 'www.youtube.com', 'm.youtube.com', 'music.youtube.com'):
        if url.path == '/watch':
            video_id = parse_qs(url.query).get('v', [None])[0]
        elif len(partes) >= 2 and partes[0] in ('shorts', 'embed', 'live'):
            video_id = partes[1]
    if not video_id or not re.fullmatch(r'[\w-]{11}', video_id):
        raise ValueError('Informe um ID ou link válido de vídeo do YouTube.')
    return video_id


def tempo(segundos):
    segundos = int(segundos)
    return f'{segundos // 3600:02d}:{segundos % 3600 // 60:02d}:{segundos % 60:02d}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('video', nargs='?', default=VIDEO_PADRAO, help='URL ou ID do vídeo')
    parser.add_argument('-o', '--saida', help='Caminho do arquivo de saída (.md)')
    parser.add_argument('--idiomas', default='pt,pt-BR,en', help='Idiomas em ordem de preferência (padrão: pt,pt-BR,en)')
    parser.add_argument('--tempos', action='store_true', help='Inclui o horário de cada trecho')
    args = parser.parse_args()

    try:
        video_id = extrair_id(args.video)
        idiomas = [idioma.strip() for idioma in args.idiomas.split(',') if idioma.strip()]
        if not idiomas:
            raise ValueError('Informe ao menos um idioma.')
        try:
            from youtube_transcript_api import YouTubeTranscriptApi
        except ImportError:
            print("Instale a dependência: python3 -m pip install 'youtube-transcript-api>=1.0,<2'", file=sys.stderr)
            return 1
        trechos = YouTubeTranscriptApi().fetch(video_id, languages=idiomas)
        linhas = []
        for trecho in trechos:
            texto = ' '.join(trecho.text.split())
            if texto:
                linhas.append(f'[{tempo(trecho.start)}] {texto}' if args.tempos else texto)
        if not linhas:
            raise ValueError('A legenda encontrada está vazia.')
        arquivo = Path(args.saida or f'transcricao_{video_id}.md')
        arquivo.write_text(f'# Transcrição do vídeo {video_id}\n\n' + '\n\n'.join(linhas) + '\n', encoding='utf-8')
        print(f'Transcrição salva em: {arquivo.resolve()}')
        print(f'Idioma: {trechos.language} ({trechos.language_code}); '
              f'{"legenda automática" if trechos.is_generated else "legenda manual"}.')
        return 0
    except Exception as erro:
        print(f'Não foi possível obter a transcrição: {erro}', file=sys.stderr)
        print('Verifique o link, os idiomas e se o vídeo disponibiliza legendas.', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
