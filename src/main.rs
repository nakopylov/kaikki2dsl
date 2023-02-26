use serde::Deserialize;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{fs::File, path::Path};

#[derive(Deserialize)]
struct Entry {
    word: String,
    pos: Option<String>,
    senses: Vec<Sense>,
    sounds: Option<Vec<Sound>>,
}

#[derive(Deserialize)]
struct Sense {
    glosses: Option<Vec<String>>,
    examples: Option<Vec<Example>>,
}

#[derive(Deserialize)]
struct Example {
    text: String,
    // english: Option<String>,
}

#[derive(Deserialize)]
struct Sound {
    ipa: Option<String>,
}

fn build_preable(name: &str, index: &str, contents: &str) -> String {
    // GoldenDict requires UTF-8 with BOM
    let preamble = format!(
        "\u{FEFF}\
        #NAME \"{}\"\n\
        #INDEX_LANGUAGE \"{}\"\n\
        #CONTENTS_LANGUAGE \"{}\"\n\n",
        name, index, contents
    );
    preamble
}

fn tab_tag(tag: &str, line: &String) -> String {
    let tagged_line = format!("\t[{0}]{1}[/{0}]\n", tag, line);
    tagged_line
}

fn build_headword(word: &String) -> String {
    return format!("{}\n", word);
}

fn build_labels(label: &String) -> String {
    return tab_tag("p", label);
}

fn build_transcription(ipa: &String) -> String {
    return tab_tag("t", ipa);
}

///
/// Well, this may be written more elegantly
///
fn build_trn(num: Option<usize>, gloss: &String) -> String {
    return if !gloss.contains("\n") {
        let line = match num {
            None => tab_tag("trn", gloss),
            Some(num) => {
                format!("\t{2}) [{0}]{1}[/{0}]\n", "trn", gloss, num + 1)
            }
        };
        line
    } else {
        let without_newline = gloss.as_str().replace("\n", "\n\t");
        let line = match num {
            None => format!("\t{}\n", without_newline),
            Some(num) => {
                format!("\t{1}) {0}\n", without_newline, num + 1)
            }
        };
        line
    };
}

fn build_ex(example: &String) -> String {
    return tab_tag("ex", example);
}

fn write_entry(
    output: &mut BufWriter<File>,
    entry: Entry,
) -> Result<(), Box<dyn std::error::Error>> {
    write!(output, "{}", build_headword(&entry.word))?;
    match entry.pos {
        Some(pos) => write!(output, "{}", build_labels(&pos))?,
        None => (),
    };
    match entry.sounds {
        Some(sounds) => {
            for sound in sounds {
                match sound.ipa {
                    Some(ipa) => write!(output, "{}", build_transcription(&ipa))?,
                    None => (),
                };
            }
        }
        None => (),
    }
    for (k, sense) in entry.senses.into_iter().enumerate() {
        match sense.glosses {
            Some(glosses) => {
                for gloss in glosses {
                    write!(output, "{}", build_trn(Some(k), &gloss))?;
                    match sense.examples {
                        Some(ref examples) => {
                            for ex in examples {
                                write!(output, "{}", build_ex(&ex.text))?;
                            }
                        }
                        None => (),
                    }
                }
            }
            None => (),
        };
    }
    Ok(())
}

fn jsonperline<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let output = File::create("./hbs.dsl").expect("Unable to create file");
    let mut output = BufWriter::new(output);
    let preamble = build_preable("Wiktionary (Hbs-Eng)", "SerbianCyrillic", "English");
    write!(output, "{}", preamble)?;
    let reader = BufReader::new(File::open(path)?);
    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line?.as_str()).unwrap();
        write_entry(&mut output, entry)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = jsonperline("./words.json");
    Ok(())
}
