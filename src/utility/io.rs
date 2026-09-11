// common/src/utility/io.rs

use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 指定されたパスのRONファイルを読み込み、対応する型に変換する
pub fn load_ron<P, T>(path: P) -> Result<T, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    // 絶対パスで直接指定してみる（または相対パスを調整）
    //// Debug space
    // let path = "src/data/character.ron";
    // let content = std::fs::read_to_string(path).expect("ファイルが読めません");
    // println!("--- 読み込んだファイルの中身 ---\n{}", content);
    // println!("実行時のカレントディレクト: {:?}", std::env::current_dir());
    // println!("ファイルの存在チェック: {}", path.exists());
    ////

    // 1. ファイルを開く
    let mut file = File::open(path)?;
    let mut content = String::new();

    // 2. テキストとして読み込む
    file.read_to_string(&mut content)?;

    // 3. RON文字列をRustの型 T に変換する
    let data = ron::from_str(&content)?;

    Ok(data)
}
