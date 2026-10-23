//! O que o script Node usava da linguagem e precisa valer igual aqui: `Number(texto)`,
//! `encodeURIComponent`, `texto.replace('#', ...)`, `path.join` e `Buffer.from(..., 'base64')`.

/// `Number(texto)`: espaços nas pontas não contam, texto vazio é 0 e o que não é número é NaN.
pub fn numero_do_texto(texto: &str) -> f64 {
    let limpo = texto.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
    if limpo.is_empty() {
        return 0.0;
    }
    for (prefixo, base) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digitos) = limpo.strip_prefix(prefixo) {
            if digitos.is_empty() || !digitos.chars().all(|c| c.is_digit(base)) {
                return f64::NAN;
            }
            return digitos.chars().fold(0.0, |total, c| {
                total * base as f64 + c.to_digit(base).unwrap() as f64
            });
        }
    }
    let sem_sinal = limpo.strip_prefix(['+', '-']).unwrap_or(limpo);
    if sem_sinal == "Infinity" {
        return if limpo.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    if !decimal_valido(sem_sinal) {
        return f64::NAN;
    }
    limpo.parse().unwrap_or(f64::NAN)
}

/// `dígitos[.dígitos]` ou `.dígitos`, com expoente opcional.
fn decimal_valido(texto: &str) -> bool {
    let (base, expoente) = match texto.find(['e', 'E']) {
        Some(posicao) => (&texto[..posicao], Some(&texto[posicao + 1..])),
        None => (texto, None),
    };
    let (inteira, fracao) = base.split_once('.').unwrap_or((base, ""));
    let so_digitos = |parte: &str| parte.chars().all(|c| c.is_ascii_digit());
    if !so_digitos(inteira) || !so_digitos(fracao) || (inteira.is_empty() && fracao.is_empty()) {
        return false;
    }
    match expoente {
        None => true,
        Some(expoente) => {
            let digitos = expoente.strip_prefix(['+', '-']).unwrap_or(expoente);
            !digitos.is_empty() && so_digitos(digitos)
        }
    }
}

/// `Array.from({ length: numero }).length`: NaN e negativos viram 0, frações são truncadas.
/// `None` quando o JavaScript recusaria o tamanho (RangeError).
pub fn tamanho_de_lista(numero: f64) -> Option<usize> {
    if numero.is_nan() || numero <= 0.0 {
        return Some(0);
    }
    let inteiro = numero.floor();
    (inteiro <= u32::MAX as f64).then_some(inteiro as usize)
}

/// `encodeURIComponent(texto)`.
pub fn codificar_componente(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    for byte in texto.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            saida.push(byte as char);
        } else {
            saida.push_str(&format!("%{byte:02X}"));
        }
    }
    saida
}

/// `texto.replace(procurado, substituto)` com um texto como padrão: troca só a primeira
/// ocorrência e entende `$$`, `$&`, `` $` `` e `$'` no substituto.
pub fn trocar_primeiro(texto: &str, procurado: &str, substituto: &str) -> String {
    let Some(posicao) = texto.find(procurado) else {
        return texto.to_string();
    };
    let antes = &texto[..posicao];
    let depois = &texto[posicao + procurado.len()..];
    let mut troca = String::new();
    let mut caracteres = substituto.chars().peekable();
    while let Some(caractere) = caracteres.next() {
        if caractere != '$' {
            troca.push(caractere);
            continue;
        }
        match caracteres.peek() {
            Some('$') => troca.push('$'),
            Some('&') => troca.push_str(procurado),
            Some('`') => troca.push_str(antes),
            Some('\'') => troca.push_str(depois),
            _ => {
                troca.push('$');
                continue;
            }
        }
        caracteres.next();
    }
    format!("{antes}{troca}{depois}")
}

/// `path.join(a, b)` no POSIX: junta com `/` e normaliza `.`, `..` e barras repetidas.
pub fn juntar_caminho(partes: &[&str]) -> String {
    let juntado = partes
        .iter()
        .filter(|parte| !parte.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("/");
    if juntado.is_empty() {
        return ".".to_string();
    }
    normalizar_caminho(&juntado)
}

fn normalizar_caminho(caminho: &str) -> String {
    let absoluto = caminho.starts_with('/');
    let barra_no_fim = caminho.ends_with('/');
    let mut segmentos: Vec<&str> = Vec::new();
    for segmento in caminho.split('/') {
        match segmento {
            "" | "." => {}
            ".." => {
                if segmentos.last().is_some_and(|ultimo| *ultimo != "..") {
                    segmentos.pop();
                } else if !absoluto {
                    segmentos.push("..");
                }
            }
            outro => segmentos.push(outro),
        }
    }
    let mut saida = segmentos.join("/");
    if saida.is_empty() && !absoluto {
        saida.push('.');
    }
    if !saida.is_empty() && barra_no_fim {
        saida.push('/');
    }
    if absoluto {
        saida.insert(0, '/');
    }
    saida
}

/// `Buffer.from(texto, 'base64')`: aceita também `-` e `_`, ignora o que não é base64 e para
/// no primeiro `=`.
pub fn decodificar_base64(texto: &str) -> Vec<u8> {
    let valor = |byte: u8| -> Option<u32> {
        match byte {
            b'A'..=b'Z' => Some((byte - b'A') as u32),
            b'a'..=b'z' => Some((byte - b'a') as u32 + 26),
            b'0'..=b'9' => Some((byte - b'0') as u32 + 52),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            _ => None,
        }
    };
    let mut saida = Vec::with_capacity(texto.len() / 4 * 3);
    let mut acumulado = 0u32;
    let mut bits = 0;
    for byte in texto.bytes() {
        if byte == b'=' {
            break;
        }
        let Some(seis_bits) = valor(byte) else {
            continue;
        };
        acumulado = (acumulado << 6) | seis_bits;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            saida.push((acumulado >> bits) as u8);
            acumulado &= (1 << bits) - 1;
        }
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn paralelo_como_no_node() {
        let tamanho = |texto: &str| tamanho_de_lista(numero_do_texto(texto));
        assert_eq!(tamanho("6"), Some(6));
        assert_eq!(tamanho(" 3 "), Some(3));
        assert_eq!(tamanho("2.7"), Some(2));
        assert_eq!(tamanho("0x10"), Some(16));
        assert_eq!(tamanho("abc"), Some(0));
        assert_eq!(tamanho("-4"), Some(0));
        assert_eq!(tamanho("1e10"), None);
        assert_eq!(tamanho("Infinity"), None);
    }

    #[test]
    fn codifica_como_encode_uri_component() {
        assert_eq!(codificar_componente("turno-do-caixa"), "turno-do-caixa");
        assert_eq!(
            codificar_componente("a b/ç?&=#!~*'()"),
            "a%20b%2F%C3%A7%3F%26%3D%23!~*'()"
        );
    }

    #[test]
    fn troca_so_o_primeiro_e_entende_cifrao() {
        assert_eq!(
            trocar_primeiro("?a=1#/x#y", "#", "&idioma=en#"),
            "?a=1&idioma=en#/x#y"
        );
        assert_eq!(trocar_primeiro("?a=1", "#", "&idioma=en#"), "?a=1");
        assert_eq!(
            trocar_primeiro("a#b", "#", "[$&|$`|$'|$$|$1]"),
            "a[#|a|b|$|$1]b"
        );
    }

    #[test]
    fn junta_como_path_join() {
        assert_eq!(
            juntar_caminho(&["prints", "rot/01-a.png"]),
            "prints/rot/01-a.png"
        );
        assert_eq!(
            juntar_caminho(&["./prints/", "rot/x.png"]),
            "prints/rot/x.png"
        );
        assert_eq!(juntar_caminho(&["/tmp//p", "../q/./x.png"]), "/tmp/q/x.png");
        assert_eq!(juntar_caminho(&["p", "../../x"]), "../x");
        assert_eq!(juntar_caminho(&["", ""]), ".");
    }

    #[test]
    fn base64_como_buffer_from() {
        assert_eq!(decodificar_base64("aGVsbG8="), b"hello");
        assert_eq!(decodificar_base64("aGVs\nbG8"), b"hello");
        assert_eq!(decodificar_base64(""), b"");
    }
}
