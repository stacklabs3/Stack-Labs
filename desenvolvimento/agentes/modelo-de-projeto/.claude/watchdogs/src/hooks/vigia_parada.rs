// Watchdog Stop: o turno só termina quando cada ticket do wayfinder resolvido nesta sessão
// tem seu ponteiro em "Decisions so far" no mapa.

use crate::comum::*;
use regex::Regex;
use serde_json::json;

pub fn rodar() -> i32 {
    let entrada_hook = ler_entrada_do_hook();
    if watchdogs_desligados() || entrada_hook["stop_hook_active"].as_bool().unwrap_or(false) {
        return 0;
    }
    let diretorio_projeto = diretorio_do_projeto(&entrada_hook);
    if etapa_de_adocao(&diretorio_projeto) == "observacao" {
        return 0;
    }
    let id_sessao = texto(&entrada_hook["session_id"]);
    let mut estado_sessao = ler_estado_sessao(&diretorio_projeto, id_sessao);
    let tem_pendente = estado_sessao["ticketsWayfinderResolvidos"]
        .as_array()
        .is_some_and(|tickets| tickets.iter().any(|ticket| !ticket["ponteiroNoMapaVerificado"].as_bool().unwrap_or(false)));
    if !tem_pendente {
        return 0;
    }

    let mut pendencias_encontradas: Vec<String> = Vec::new();
    let secao_de_decisoes = Regex::new(r"(?i)##\s*Decisions so far").expect("regex válida");
    let proxima_secao = Regex::new(r"\n##\s").expect("regex válida");
    for ticket_pendente in estado_sessao["ticketsWayfinderResolvidos"].as_array_mut().into_iter().flatten() {
        if ticket_pendente["ponteiroNoMapaVerificado"].as_bool().unwrap_or(false) {
            continue;
        }
        let numero_ticket = ticket_pendente["numero"].as_i64().unwrap_or(0);
        let titulo_ticket = texto(&ticket_pendente["titulo"]).to_string();
        let verificacao = (|| -> Result<Option<String>, String> {
            let ticket_completo = buscar_issue(&diretorio_projeto, numero_ticket)?;
            if ticket_completo.estado != "closed" {
                return Ok(None); // fechamento falhou ou ainda não aconteceu
            }
            let Some(numero_mapa) = numero_do_mapa(&diretorio_projeto, &ticket_completo) else {
                return Ok(Some(format!(
                    "\"{titulo_ticket}\" (#{numero_ticket}) não está ligado a nenhum mapa (sub-issue ou \"Part of #N\")."
                )));
            };
            let mapa_do_ticket = buscar_issue(&diretorio_projeto, numero_mapa)?;
            let apos_decisoes = secao_de_decisoes.splitn(&mapa_do_ticket.corpo, 2).nth(1).unwrap_or("");
            let secao_decisoes = proxima_secao.split(apos_decisoes).next().unwrap_or("");
            let ponteiro = Regex::new(&format!(r"(issues/{numero_ticket}\b|#{numero_ticket}\b)")).expect("regex válida");
            if ponteiro.is_match(secao_decisoes) {
                return Ok(Some(String::new()));
            }
            Ok(Some(format!("o mapa \"{}\" (#{numero_mapa}) não tem o ponteiro de \"{titulo_ticket}\" (#{numero_ticket}) em \"Decisions so far\". Acrescente: - [{titulo_ticket}](link): <gist de uma linha>", mapa_do_ticket.titulo)))
        })();
        match verificacao {
            Ok(Some(pendencia)) if pendencia.is_empty() => ticket_pendente["ponteiroNoMapaVerificado"] = true.into(),
            Ok(Some(pendencia)) => pendencias_encontradas.push(pendencia),
            Ok(None) => {}
            Err(_) => pendencias_encontradas.push(format!(
                "não consegui verificar \"{titulo_ticket}\" (#{numero_ticket}) no GitHub. Confira o mapa manualmente."
            )),
        }
    }
    gravar_estado_sessao(&diretorio_projeto, id_sessao, &estado_sessao);

    if !pendencias_encontradas.is_empty() {
        let motivo = format!("[watchdog vigia-parada] Resolução do wayfinder incompleta:\n- {}\nDepois, gradue a névoa que a resposta esclareceu e atualize \"Not yet specified\".", pendencias_encontradas.join("\n- "));
        print!("{}", json!({ "decision": "block", "reason": motivo }));
    }
    0
}
