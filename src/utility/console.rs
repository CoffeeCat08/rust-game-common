// src/utility/console.rs

use std::io::Write;

pub struct ConsoleMap {
    pub width: usize, // x軸
    //  hight: usize, // y軸 2Dなら使用しない。
    pub depth: usize,
    pub grid: Vec<Vec<char>>,
    dirty_cells: Vec<(usize, usize)>,
    pub offset_x: usize,
    pub offset_z: usize,
}

impl ConsoleMap {
    pub fn new(width: usize, depth: usize, offset_x: usize, offset_z: usize) -> Self {
        Self {
            width,
            depth,
            grid: vec![vec![' '; width]; depth],
            dirty_cells: Vec::new(),
            // ANSIコードは1スタートなので1未満を防止
            offset_x: offset_x.max(1),
            offset_z: offset_z.max(1),
        }
    }

    pub fn input(&mut self, x: usize, z: usize, key: char) {
        if x < self.width && z < self.depth {
            if self.grid[z][x] != key {
                self.grid[z][x] = key;
                self.dirty_cells.push((x, z));
            }
        }
    }

    pub fn output(&self) {
        for line in &self.grid {
            for word in line {
                print!(":{}", word);
            }
            println!();
        }
    }

    pub fn flush(&mut self) {
        while let Some((x, z)) = self.dirty_cells.pop() {
            let ch = self.grid[z][x];
            let target_col = self.offset_x + x;
            let target_row = self.offset_z + z;

            // カーソル移動して描画
            print!("\x1b[{};{}H{}", target_row, target_col, ch);
        }
        let _ = std::io::stdout().flush();
    }

    /// 盤面をクリア（次回flush時に全削除が反映される）
    pub fn clear(&mut self) {
        for z in 0..self.depth {
            for x in 0..self.width {
                self.input(x, z, ' ');
            }
        }
    }

    pub fn clear_screen(&self) {
        // \x1b[2J = 画面クリア, \x1b[H = カーソルを(1,1)に移動
        print!("\x1b[2J\x1b[H");
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}

pub struct ConsoleText {
    pub offset_x: usize,
    pub offset_z: usize,
}

impl ConsoleText {
    pub fn new(x: usize, z: usize) -> Self {
        Self {
            offset_x: x,
            offset_z: z,
        }
    }

    pub fn status_view(&self, label: &str, max_amount: f32, amount: f32) {
        let persent = amount / max_amount;
        let count = (persent * 10.0) as usize;

        let mut bar = String::new();
        bar.push('[');
        for i in 0..10 {
            if i < count {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }
        bar.push(']');

        println!(
            "\x1b[{};{}H{}: {} {}%\n",
            self.offset_z,
            self.offset_x,
            label,
            bar,
            persent * 100.0
        );

        let _ = std::io::stdout().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_view() {
        let width: usize = 5;
        let depth: usize = 5;
        let offset_x = 1;
        let offset_z = 1;
        let mut test = ConsoleMap::new(width, depth, offset_x, offset_z);
        for h in 0..depth {
            for i in 0..width {
                test.input(i, h, '*');
            }
        }
        let ans = vec![
            vec!['*', '*', '*', '*', '*'],
            vec!['*', '*', '*', '*', '*'],
            vec!['*', '*', '*', '*', '*'],
            vec!['*', '*', '*', '*', '*'],
            vec!['*', '*', '*', '*', '*'],
        ];
        assert_eq!(ans, test.grid);
    }

    #[test]
    fn test_status_view() {
        let amount = 72.8;
        let max_amount = 100.0;
        let test = status_view(max_amount, amount);
        println!("\n [DEBUG_PRINT] -> {}", test);
        assert_eq!("[███████░░░] 72.8%", test);
    }
}
