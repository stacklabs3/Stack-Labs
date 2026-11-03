#!/usr/bin/env python3
"""Compara três formas de mostrar um teste à revisao-por-modelo (issue stacklabs1/Stack-Labs#53).

Variantes:
  atual      o trecho que o watchdog envia hoje (cabeçalho colado, declaração e corpo).
  auxiliares o trecho atual mais as declarações de topo do arquivo que ele usa (com as que
             elas usam) e os beforeEach/beforeAll dos describes que o envolvem. Sem o título
             do describe.
  arquivo    o arquivo de teste inteiro, com a linha do teste a julgar.

Gabarito: para cada teste, uma cópia com o RK trocado por outro risco do registro. A resposta
esperada é exercita=false. Uma variante que deixa o RK trocado passar está leniente.

Roda o `claude -p` com as mesmas flags do watchdog e sem cache. Só JS e TS (it/test).

Uso, dentro do checkout do projeto (ex.: organiza-ai):
  python3 comparar.py --projeto . --saida .scratch/comparacao src/**/*.test.ts
  python3 comparar.py --simular ...   # monta os prompts e mede o tamanho, sem chamar o modelo
"""

import argparse
import csv
import json
import random
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# Copiados de .claude/watchdogs/src/revisao_por_modelo.rs. Se o watchdog mudar, atualize aqui.
MODELO_PADRAO = "claude-haiku-5-5"
TETO_PADRAO_USD = "0.05"
LINHAS_MAXIMAS_DO_TRECHO = 150
ESQUEMA = '{"type":"object","properties":{"exercita":{"type":"boolean"},"motivo":{"type":"string"}},"required":["exercita","motivo"]}'
INSTRUCAO_BASE = (
    "Você revisa um teste automatizado contra o risco que ele diz cobrir. O teste exercita o risco "
    "quando falharia se o risco acontecesse: ele monta o cenário do risco e afirma o resultado que o risco quebraria. "
    "Responda exercita=false quando o teste só cita o risco no nome ou num comentário, afirma outra coisa (um getter, "
    "o valor que acabou de montar, uma chamada de mock) ou passaria mesmo com o risco acontecendo. "
)
FIM_DA_INSTRUCAO = {
    "atual": "Julgue só pelo trecho abaixo.",
    "auxiliares": "Julgue pelo teste e pelo contexto do arquivo abaixo dele.",
    "arquivo": "Julgue o teste indicado, usando o resto do arquivo como contexto.",
}
MOTIVO = " Em motivo, uma ou duas frases em português que digam o que o teste afirma e o que faltaria."

DECLARACAO_JS = re.compile(r"^\s*(?:it|test)\b[\w.]*\s*\(")
GANCHO_JS = re.compile(r"^\s*(?:beforeEach|beforeAll|afterEach|afterAll)\s*\(")
DESCRIBE_JS = re.compile(r"^\s*describe\b[\w.]*\s*\(")
CITACAO = re.compile(r"(?i)rk(\d+)")
ID_NA_TABELA = re.compile(r"^RK\d+$")
NOME_JS = re.compile(r"""\b(?:it|test)\b[\w.]*\s*\(\s*["'`]([^"'`]*)""")
DECLARACAO_DE_TOPO = re.compile(r"^(?:export\s+)?(?:default\s+)?(?:async\s+)?(?:const|let|var|function\*?|class|type|interface|enum)\s+([A-Za-z_$][\w$]*)")
IDENTIFICADOR = re.compile(r"[A-Za-z_$][\w$]*")


def recuo(linha):
    return len(linha) - len(linha.lstrip())


def eh_cabecalho(linha):
    return linha.lstrip().startswith(("//", "/*", "*", "#", "@"))


def riscos_citados(texto):
    citados = []
    for achado in CITACAO.finditer(texto):
        anterior = texto[achado.start() - 1] if achado.start() > 0 else None
        maiuscula = texto[achado.start()] == "R"
        if anterior is None or not anterior.isalnum() or (maiuscula and anterior.islower()):
            citados.append(f"RK{achado.group(1)}")
    return citados


def ler_registro(caminho):
    riscos = {}
    for linha in caminho.read_text(encoding="utf-8").splitlines():
        partes = linha.split("|")
        if len(partes) < 3:
            continue
        celula = partes[1].strip()
        if ID_NA_TABELA.match(celula):
            riscos.setdefault(celula, linha.strip())
    return riscos


def fim_do_bloco(linhas, inicio, maximo=None):
    """Índice (exclusivo) da linha que fecha o bloco aberto a partir de `inicio`, contando chaves."""
    limite = len(linhas) if maximo is None else min(len(linhas), inicio + maximo)
    profundidade, abriu = 0, False
    for indice in range(inicio, limite):
        for letra in linhas[indice]:
            if letra == "{":
                profundidade += 1
                abriu = True
            elif letra == "}":
                profundidade -= 1
        if abriu and profundidade <= 0:
            return indice + 1
        if not abriu and indice > inicio and linhas[indice].rstrip().endswith((";", ")")):
            return indice + 1
    return limite


def inicio_com_cabecalho(linhas, indice):
    while indice > 0 and eh_cabecalho(linhas[indice - 1]):
        indice -= 1
    return indice


def trecho_atual(linhas, indice):
    """Porta de trecho_do_teste (revisao_por_modelo.rs), só o caminho com chaves."""
    inicio = inicio_com_cabecalho(linhas, indice)
    limite = min(indice + LINHAS_MAXIMAS_DO_TRECHO, len(linhas))
    fim = limite
    if any("{" in linha for linha in linhas[indice:min(limite, indice + 6)]):
        profundidade, abriu = 0, False
        for posicao in range(indice, limite):
            for letra in linhas[posicao]:
                if letra == "{":
                    profundidade += 1
                    abriu = True
                elif letra == "}":
                    profundidade -= 1
            if abriu and profundidade <= 0:
                fim = posicao + 1
                break
    return inicio, fim


def ancestrais(linhas, indice):
    """Linhas (índices) dos blocos que envolvem o teste, do mais interno para fora."""
    encontrados, nivel = [], recuo(linhas[indice])
    for acima in range(indice - 1, -1, -1):
        if nivel == 0:
            break
        candidata = linhas[acima]
        aparada = candidata.lstrip()
        if not aparada or aparada.startswith(("}", ")", "]")) or recuo(candidata) >= nivel:
            continue
        nivel = recuo(candidata)
        encontrados.append(acima)
    return encontrados


def riscos_do_teste(linhas, indice):
    """Mesma ideia de testes_do_arquivo (riscos.rs): declaração, cabeçalho e blocos que envolvem."""
    trechos = [linhas[indice]]
    abaixo = indice
    while abaixo + 1 < len(linhas) and eh_cabecalho(linhas[abaixo]):
        abaixo += 1
        trechos.append(linhas[abaixo])
    for posicao in [indice] + ancestrais(linhas, indice):
        trechos.append(linhas[posicao])
        trechos.extend(linhas[inicio_com_cabecalho(linhas, posicao):posicao])
    return sorted(set(citado for trecho in trechos for citado in riscos_citados(trecho)))


def declaracoes_de_topo(linhas):
    """nome -> (início, fim) das declarações sem recuo (auxiliares, constantes, tipos)."""
    declaracoes = {}
    for indice, linha in enumerate(linhas):
        if recuo(linha) != 0:
            continue
        achado = DECLARACAO_DE_TOPO.match(linha)
        if achado and not DESCRIBE_JS.match(linha):
            declaracoes[achado.group(1)] = (inicio_com_cabecalho(linhas, indice), fim_do_bloco(linhas, indice))
    return declaracoes


def importacoes(linhas):
    return [linha for linha in linhas if linha.startswith("import ") or (linha.startswith("const ") and "require(" in linha)]


def contexto_auxiliar(linhas, indice, inicio, fim):
    """Auxiliares de topo usados pelo teste (até 3 níveis) e ganchos dos describes que o envolvem."""
    envolventes = set(ancestrais(linhas, indice))
    ganchos = []
    for posicao, linha in enumerate(linhas[:indice]):
        if not GANCHO_JS.match(linha):
            continue
        dono = ancestrais(linhas, posicao)
        if not dono or dono[0] in envolventes:
            ganchos.append((inicio_com_cabecalho(linhas, posicao), fim_do_bloco(linhas, posicao)))
    declaracoes = declaracoes_de_topo(linhas)
    escolhidas = set()
    a_olhar = "\n".join(linhas[inicio:fim]) + "\n" + "\n".join("\n".join(linhas[comeco:final]) for comeco, final in ganchos)
    for _ in range(3):
        novos = {nome for nome in set(IDENTIFICADOR.findall(a_olhar)) if nome in declaracoes and nome not in escolhidas}
        if not novos:
            break
        escolhidas |= novos
        a_olhar = "\n".join("\n".join(linhas[declaracoes[nome][0]:declaracoes[nome][1]]) for nome in novos)
    blocos = sorted({declaracoes[nome] for nome in escolhidas})
    partes = []
    if importacoes(linhas):
        partes.append("// importações do arquivo\n" + "\n".join(importacoes(linhas)))
    for comeco, final in blocos:
        partes.append("\n".join(linhas[comeco:final]))
    for comeco, final in ganchos:
        partes.append("// gancho do describe que envolve o teste\n" + "\n".join(linhas[comeco:final]))
    return "\n\n".join(partes)


def montar_prompt(variante, arquivo, linha, linhas_dos_riscos, linhas, indice):
    instrucao = INSTRUCAO_BASE + FIM_DA_INSTRUCAO[variante] + MOTIVO
    riscos = "\n".join(linhas_dos_riscos)
    inicio, fim = trecho_atual(linhas, indice)
    trecho = "\n".join(linhas[inicio:fim])
    if variante == "atual":
        return f"{instrucao}\n\nRisco citado (linha de docs/riscos.md):\n{riscos}\n\nTeste ({arquivo}:{linha}):\n```\n{trecho}\n```"
    if variante == "auxiliares":
        contexto = contexto_auxiliar(linhas, indice, inicio, fim) or "(nenhum auxiliar de fora)"
        return (f"{instrucao}\n\nRisco citado (linha de docs/riscos.md):\n{riscos}\n\nTeste ({arquivo}:{linha}):\n```\n{trecho}\n```"
                f"\n\nContexto do arquivo usado pelo teste:\n```\n{contexto}\n```")
    numerado = "\n".join(f"{posicao + 1:>4}| {texto}" for posicao, texto in enumerate(linhas))
    return (f"{instrucao}\n\nRisco citado (linha de docs/riscos.md):\n{riscos}\n\nTeste a julgar: {arquivo}:{linha} "
            f"(o bloco que começa na linha {linha}).\n\nArquivo inteiro:\n```\n{numerado}\n```")


def trocar_rk(linhas, citados, novo):
    """Troca cada RK citado pelo novo, no arquivo todo, respeitando a fronteira de riscos_citados."""
    numeros = {citado[2:] for citado in citados}

    def trocar(achado):
        texto, inicio = achado.string, achado.start()
        anterior = texto[inicio - 1] if inicio > 0 else None
        fronteira = anterior is None or not anterior.isalnum() or (texto[inicio] == "R" and anterior.islower())
        if fronteira and achado.group(1) in numeros:
            return achado.group(0)[:2] + novo[2:]
        return achado.group(0)

    return [CITACAO.sub(trocar, linha) for linha in linhas]


def perguntar(modelo, teto, limite, prompt, raiz):
    comando = ["claude", "-p", "--setting-sources", "", "--strict-mcp-config", "--model", modelo, "--tools", "",
               "--no-session-persistence", "--output-format", "json", "--json-schema", ESQUEMA,
               "--max-budget-usd", teto, prompt]
    try:
        saida = subprocess.run(comando, cwd=raiz, capture_output=True, text=True, timeout=limite,
                               stdin=subprocess.DEVNULL, env={**__import__("os").environ, "STACKLABS_WATCHDOGS": "desligado"})
    except subprocess.TimeoutExpired:
        return {"erro": f"sem resposta em {limite} s"}
    try:
        resposta = json.loads(saida.stdout.strip())
    except json.JSONDecodeError:
        return {"erro": (saida.stdout + " " + saida.stderr).strip()[:200]}
    if resposta.get("is_error"):
        return {"erro": str(resposta.get("result", "erro sem mensagem"))[:200], "custo": resposta.get("total_cost_usd")}
    estruturada = resposta.get("structured_output") or {}
    if not isinstance(estruturada.get("exercita"), bool):
        return {"erro": "resposta sem structured_output.exercita", "custo": resposta.get("total_cost_usd")}
    return {"exercita": estruturada["exercita"], "motivo": estruturada.get("motivo", "").strip(),
            "custo": resposta.get("total_cost_usd")}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("arquivos", nargs="+", help="arquivos de teste, relativos a --projeto")
    parser.add_argument("--projeto", default=".", help="raiz do projeto (onde fica docs/riscos.md)")
    parser.add_argument("--riscos", default="docs/riscos.md")
    parser.add_argument("--modelo", default=MODELO_PADRAO)
    parser.add_argument("--teto", default=TETO_PADRAO_USD, help="teto por chamada, em US$ (o mesmo do watchdog)")
    parser.add_argument("--limite", type=int, default=120, help="segundos por chamada")
    parser.add_argument("--variantes", default="atual,auxiliares,arquivo")
    parser.add_argument("--sem-gabarito", action="store_true")
    parser.add_argument("--simultaneas", type=int, default=4)
    parser.add_argument("--semente", type=int, default=53, help="escolha do RK trocado no gabarito")
    parser.add_argument("--saida", default=".scratch/comparacao-revisao-por-modelo")
    parser.add_argument("--simular", action="store_true", help="só monta os prompts e mede o tamanho")
    argumentos = parser.parse_args()

    raiz = Path(argumentos.projeto).resolve()
    registro = ler_registro(raiz / argumentos.riscos)
    if not registro:
        sys.exit(f"nenhum RK em {raiz / argumentos.riscos}")
    variantes = [variante.strip() for variante in argumentos.variantes.split(",")]
    sorteio = random.Random(argumentos.semente)
    saida = raiz / argumentos.saida
    (saida / "prompts").mkdir(parents=True, exist_ok=True)

    pedidos = []
    for arquivo in argumentos.arquivos:
        linhas = (raiz / arquivo).read_text(encoding="utf-8").splitlines()
        for indice, linha in enumerate(linhas):
            if not DECLARACAO_JS.match(linha):
                continue
            citados = [citado for citado in riscos_do_teste(linhas, indice) if citado in registro]
            if not citados:
                continue
            achado_do_nome = NOME_JS.search(linha)
            nome = achado_do_nome.group(1) if achado_do_nome else ""
            casos = [("real", citados, linhas)]
            outros = sorted(set(registro) - set(citados))
            if not argumentos.sem_gabarito and outros:
                trocado = sorteio.choice(outros)
                casos.append(("gabarito", [trocado], trocar_rk(linhas, citados, trocado)))
            for tipo, riscos, texto in casos:
                for variante in variantes:
                    prompt = montar_prompt(variante, arquivo, indice + 1, [registro[risco] for risco in riscos], texto, indice)
                    pedidos.append({"variante": variante, "tipo": tipo, "arquivo": arquivo, "linha": indice + 1,
                                    "nome": nome, "riscos_originais": " ".join(citados), "risco_no_prompt": " ".join(riscos),
                                    "caracteres": len(prompt), "prompt": prompt})

    for numero, pedido in enumerate(pedidos):
        caminho = saida / "prompts" / f"{numero:04d}-{pedido['variante']}-{pedido['tipo']}.txt"
        caminho.write_text(pedido["prompt"], encoding="utf-8")
        pedido["arquivo_do_prompt"] = caminho.name

    if argumentos.simular:
        for variante in variantes:
            tamanhos = [pedido["caracteres"] for pedido in pedidos if pedido["variante"] == variante]
            if tamanhos:
                print(f"{variante:<11} {len(tamanhos):>4} prompts, média {sum(tamanhos) // len(tamanhos)} caracteres, máximo {max(tamanhos)}")
        print(f"prompts em {saida / 'prompts'}")
        return

    print(f"{len(pedidos)} chamadas ao {argumentos.modelo}, {argumentos.simultaneas} por vez...", file=sys.stderr)
    with ThreadPoolExecutor(argumentos.simultaneas) as executor:
        respostas = list(executor.map(lambda pedido: perguntar(argumentos.modelo, argumentos.teto, argumentos.limite, pedido["prompt"], raiz), pedidos))

    campos = ["variante", "tipo", "arquivo", "linha", "nome", "riscos_originais", "risco_no_prompt", "exercita",
              "motivo", "erro", "custo", "caracteres", "arquivo_do_prompt"]
    with open(saida / "resultados.csv", "w", newline="", encoding="utf-8") as destino:
        escritor = csv.DictWriter(destino, fieldnames=campos, extrasaction="ignore")
        escritor.writeheader()
        for pedido, resposta in zip(pedidos, respostas):
            pedido.update(resposta)
            escritor.writerow(pedido)

    # Para a revisão humana: cada teste real, com o veredito de cada variante lado a lado.
    por_teste = {}
    for pedido in pedidos:
        if pedido["tipo"] == "real":
            por_teste.setdefault((pedido["arquivo"], pedido["linha"]), {"nome": pedido["nome"], "riscos": pedido["riscos_originais"]})[pedido["variante"]] = pedido
    with open(saida / "revisao-humana.csv", "w", newline="", encoding="utf-8") as destino:
        escritor = csv.writer(destino)
        escritor.writerow(["arquivo", "linha", "nome", "riscos"] + [f"{variante}_exercita" for variante in variantes]
                          + [f"{variante}_motivo" for variante in variantes] + ["humano_exercita (true/false)", "nota"])
        for (arquivo, linha), dados in sorted(por_teste.items()):
            escritor.writerow([arquivo, linha, dados["nome"], dados["riscos"]]
                              + [dados.get(variante, {}).get("exercita", "erro") for variante in variantes]
                              + [dados.get(variante, {}).get("motivo", dados.get(variante, {}).get("erro", "")) for variante in variantes]
                              + ["", ""])

    linhas_do_resumo = ["# Comparação da revisao-por-modelo", "",
                        f"Modelo: `{argumentos.modelo}`. Teto por chamada: US$ {argumentos.teto}. Sem cache.", "",
                        "| Variante | Testes reais | Suspeitos | Gabarito pego | Sem veredito | Custo médio (US$) | Custo máximo (US$) | Caracteres médios |",
                        "|---|---:|---:|---:|---:|---:|---:|---:|"]
    for variante in variantes:
        reais = [pedido for pedido in pedidos if pedido["variante"] == variante and pedido["tipo"] == "real"]
        gabarito = [pedido for pedido in pedidos if pedido["variante"] == variante and pedido["tipo"] == "gabarito"]
        todos = reais + gabarito
        custos = [pedido["custo"] for pedido in todos if isinstance(pedido.get("custo"), (int, float))]
        suspeitos = sum(1 for pedido in reais if pedido.get("exercita") is False)
        pegos = sum(1 for pedido in gabarito if pedido.get("exercita") is False)
        sem = sum(1 for pedido in todos if "erro" in pedido)
        linhas_do_resumo.append(
            f"| {variante} | {len(reais)} | {suspeitos} | {pegos} de {len(gabarito)} | {sem} | "
            f"{(sum(custos) / len(custos)) if custos else 0:.4f} | {max(custos) if custos else 0:.4f} | "
            f"{sum(pedido['caracteres'] for pedido in todos) // max(len(todos), 1)} |")
    linhas_do_resumo += ["", "Próximo passo: preencher `humano_exercita` em `revisao-humana.csv` e contar, por variante,",
                         "os suspeitos falsos (modelo false, humano true) e os aprovados errados (modelo true, humano false).",
                         "Um RK trocado que passa no gabarito pode ser um risco que o teste também cobre: confira o motivo."]
    (saida / "resumo.md").write_text("\n".join(linhas_do_resumo) + "\n", encoding="utf-8")
    print("\n".join(linhas_do_resumo))
    print(f"\narquivos em {saida}", file=sys.stderr)


if __name__ == "__main__":
    main()
