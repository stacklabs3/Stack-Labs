// Os hooks do Claude Code. Cada um lê a entrada JSON do stdin e responde como o Claude Code espera:
// contexto no stdout, ou exit 2 com o motivo no stderr para bloquear a ferramenta.

mod guarda_bash;
mod guarda_edicao;
mod sessao_inicio;
mod vigia_parada;
mod vigia_prompt;
mod vigia_skill;

pub fn rodar(nome_do_hook: &str) -> i32 {
    match nome_do_hook {
        "sessao-inicio" => sessao_inicio::rodar(),
        "vigia-prompt" => vigia_prompt::rodar(),
        "vigia-skill" => vigia_skill::rodar(),
        "guarda-edicao" => guarda_edicao::rodar(),
        "guarda-bash" => guarda_bash::rodar(),
        "vigia-parada" => vigia_parada::rodar(),
        _ => {
            eprintln!("hook desconhecido: {nome_do_hook} (sessao-inicio, vigia-prompt, vigia-skill, guarda-edicao, guarda-bash, vigia-parada)");
            2
        }
    }
}
