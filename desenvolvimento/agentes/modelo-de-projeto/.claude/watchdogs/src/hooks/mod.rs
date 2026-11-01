// Os hooks do Claude Code. Cada um lê a entrada JSON do stdin e responde como o Claude Code espera:
// contexto no stdout, ou exit 2 com o motivo no stderr para bloquear a ferramenta.

mod guarda_bash;
mod guarda_contexto;
mod guarda_edicao;
mod sessao_inicio;
mod vigia_agente;
mod vigia_custo;
mod vigia_parada;
mod vigia_prompt;
mod vigia_saida;
mod vigia_skill;
mod vigia_teste;

pub fn rodar(nome_do_hook: &str) -> i32 {
    match nome_do_hook {
        "sessao-inicio" => sessao_inicio::rodar(),
        "vigia-prompt" => vigia_prompt::rodar(),
        "vigia-skill" => vigia_skill::rodar(),
        "guarda-edicao" => guarda_edicao::rodar(),
        "guarda-bash" => guarda_bash::rodar(),
        "guarda-contexto" => guarda_contexto::rodar(),
        "vigia-parada" => vigia_parada::rodar(),
        "vigia-teste" => vigia_teste::rodar(),
        "vigia-custo" => vigia_custo::rodar(),
        "vigia-saida" => vigia_saida::rodar(),
        "vigia-agente" => vigia_agente::rodar(),
        _ => {
            eprintln!("hook desconhecido: {nome_do_hook} (sessao-inicio, vigia-prompt, vigia-skill, guarda-edicao, guarda-bash, guarda-contexto, vigia-parada, vigia-teste, vigia-custo, vigia-saida, vigia-agente)");
            2
        }
    }
}
