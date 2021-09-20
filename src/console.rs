use anyhow::{anyhow, Result};
use reqwest::Client;
use rustyline::Editor;
use tokio::runtime::Runtime;

use crate::feat::*;
use crate::formatter::{opencc_convert, OpenccConvertMode};

pub struct MeowdictConsole {
    pub input_s2t: bool,
    pub result_t2s: bool,
    pub client: Client,
    pub runtime: Runtime,
    pub no_color_output: bool,
}

impl MeowdictConsole {
    pub fn create_console(&mut self) {
        let mut reader = Editor::<()>::new();
        while let Ok(argument) = reader.readline("meowdict > ") {
            let argument = argument
                .trim()
                .split(' ')
                .filter(|x| x != &"")
                .collect::<Vec<&str>>();
            if !argument.is_empty() {
                let (args, words) = argument_spliter(argument);
                if let Err(e) = self.args_runner(args, words) {
                    println!("{}", e);
                }
            }
        }
    }

    fn set_console_mode(&mut self, t: &OpenccConvertMode, enable: bool) {
        match t {
            OpenccConvertMode::S2T => {
                println!(
                    "{} input mode...",
                    if enable { "Setting" } else { "Unsetting" }
                );
                self.input_s2t = enable;
            }
            OpenccConvertMode::T2S => {
                println!(
                    "{} result mode...",
                    if enable { "Setting" } else { "Unsetting" }
                );
                self.result_t2s = enable;
            }
        };
    }

    fn args_runner(&mut self, args: Vec<&str>, words_mut: Vec<&str>) -> Result<()> {
        let mut words_mut: Vec<String> = words_mut.into_iter().map(|x| x.into()).collect();
        let mut command_result_t2s = false;
        let mut command_input_s2t = false;
        let mut translation_mode = false;
        let mut jyutping_mode = false;
        for i in args {
            match i {
                "--input-s2t" => command_input_s2t = true,
                "-i" => command_input_s2t = true,
                "--result-t2s" => command_result_t2s = true,
                "-r" => command_result_t2s = true,
                "--translation" => translation_mode = true,
                "-t" => translation_mode = true,
                "--jyutping" => jyutping_mode = true,
                "-j" => jyutping_mode = true,
                "--set-mode-input-s2t" => self.set_console_mode(&OpenccConvertMode::S2T, true),
                "--set-mode-result-t2s" => self.set_console_mode(&OpenccConvertMode::T2S, true),
                "--unset-mode-input-s2t" => self.set_console_mode(&OpenccConvertMode::S2T, false),
                "--unset-mode-result-t2s" => self.set_console_mode(&OpenccConvertMode::T2S, false),
                "--unset-mode-all" => {
                    self.set_console_mode(&OpenccConvertMode::S2T, false);
                    self.set_console_mode(&OpenccConvertMode::T2S, false)
                }
                _ => return Err(anyhow!("Invaild argument: {}", i)),
            };
        }
        if self.input_s2t || command_input_s2t {
            words_mut = words_mut
                .into_iter()
                .map(|x| opencc_convert(&x, OpenccConvertMode::S2T))
                .collect::<Vec<_>>();
        }
        if translation_mode {
            if let Err(e) = search_word_to_translation_result(
                words_mut,
                &self.client,
                &self.runtime,
                self.no_color_output,
                command_result_t2s || self.result_t2s,
            ) {
                println!("{}", e);
            }
        } else if jyutping_mode {
            if let Err(e) = search_word_to_jyutping_result(
                words_mut,
                &self.client,
                &self.runtime,
                self.no_color_output,
                command_result_t2s || self.result_t2s,
            ) {
                println!("{}", e);
            }
        } else if let Err(e) = search_word_to_dict_result(
            words_mut,
            &self.client,
            &self.runtime,
            self.no_color_output,
            command_result_t2s || self.result_t2s,
        ) {
            println!("{}", e);
        }

        Ok(())
    }
}

fn argument_spliter(argument: Vec<&str>) -> (Vec<&str>, Vec<&str>) {
    let args: Vec<&str> = argument
        .clone()
        .into_iter()
        .filter(|x| x.starts_with('-'))
        .collect();
    let words: Vec<&str> = argument
        .into_iter()
        .filter(|x| !x.starts_with('-'))
        .collect();

    (args, words)
}
