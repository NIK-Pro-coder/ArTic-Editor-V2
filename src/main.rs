
use std::{env, process::{Command, exit}, fs, fs::File, io::{Read, Write}};
use macroquad::prelude::*;

// define constants
const IMPORTANT : [&str; 3] = ["Palette", "Tiles", "Sprites"];
const DEFAULT_PALETTE : [u8; 48] = [26, 28, 44, 93, 39, 93, 177, 62, 83, 239, 125, 87, 255, 205, 117, 167, 240, 112, 56, 183, 100, 37, 113, 121, 41, 54, 111, 59, 93, 201, 65, 166, 246, 115, 239, 247, 244, 244, 244, 148, 176, 194, 86, 108, 134, 51, 60, 87];
const PIX_SIZE : f32 = 4.0;
const SPR_SIDE_LENGTH : f32 = 8.0 * PIX_SIZE;
const ALL_SIDE_LENGTH : f32 = SPR_SIDE_LENGTH * 16.0;
const SCREEN_HEIGHT : f32 = 600.0;
const SCREEN_WIDTH : f32 = 800.0;
const OFF_Y : f32 = SCREEN_HEIGHT / 2.0 - 8.0 * SPR_SIDE_LENGTH;
const OFF_X : f32 = SCREEN_WIDTH - 16.0 * SPR_SIDE_LENGTH - OFF_Y;
const PALETTE_SIZE : f32 = 6.0 * PIX_SIZE;
const SELECTION_THICK : f32 = 8.0;
const EMPTY_SPR : [u8; 64] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const IMG_EXTENTIONS : [&str; 3] = ["jpg", "jpeg", "png"];

#[derive(Clone, Debug)]
struct Chunk {
    bank : u8,
    data : Vec<u8>,
    name : String,
}

fn build_chunk(c_bank: u8, c_data: &Vec<u8>, c_name: String) -> Chunk {

    // .clone() just to be sure

    Chunk{
        bank : c_bank.clone(),
        data : c_data.clone(),
        name : c_name.clone(),
    }
}

fn deconstruct_tic(path: String) -> Vec<Chunk> {
    // reading the .tic file
    let mut f = File::open(String::from(path.clone())).expect("No file found!");

    // get file size
    let size : u64 = fs::metadata(path.clone()).expect("No file found").len();

    // creating a vector to store the bytes
    let mut buf = vec![0; size as usize];

    // put the bytes into the vector
    let _ = f.read_exact(&mut buf);

    // TODO :
    // separate chunks

    let mut chunks : Vec<Chunk> = vec![];
    let mut check = 0;

    // static types are good

    let mut chunk_size : u16 = 0;
    let mut chunk_bank : u8 = 0;
    let mut chunk_type : &str = "";
    let mut chunk_data : Vec<u8> = vec![];

    for i in buf {

        // chunks follow the scheme of
        // type(5 bits) + bank(3 bits)

        chunk_type = match check {
            0 => match i & 0b00011111 {
                1 => "Tiles",
                2 => "Sprites",
                4 => "Map",
                5 => "Code",
                6 => "Flags",
                9 => "Samples",
                10 => "Waveform",
                12 => "Palette",
                14 => "Music",
                15 => "Patterns",
                17 => "Default",
                18 => "Screen",
                19 => "Binary",
                _ => "(Reserved)"
                },
            _ => chunk_type,
        };
        chunk_bank = match check {
            0 => i & 0b11100000,
            _ => chunk_bank,
        };

        // size(16 bits)

        chunk_size = match check {
            1 => i as u16,
            2 => chunk_size + ((i as u16) << 8),
            _ => chunk_size,
        };

        // reserved(8 bits)

        // actual data(size bits)

        if check == 4 {
            if chunk_size > 0 {
                chunk_size -= 1;
                chunk_data.push(i);
            } else {
                check = 0;
            }
        }

        // handle data insertion

        if check < 3 {
            // cycle state

            check += 1;
        } else if chunk_size == 0 {
            // reset state

            check = 0;

            // add chunk

            chunks.push(
                build_chunk(
                    chunk_bank,
                    &chunk_data,
                    chunk_type.into()
                )
            );
            chunk_data.clear();
        } else {
            // set state

            check = 4;
        }
    }

    chunks
}

fn extract(from: Vec<Chunk>, name: String) -> Chunk {
    for i in from {
        if i.name == name {
            return i
        }
    }

    return Chunk{
        bank : 0,
        data : vec![],
        name : name,
    }
}

fn replace(from: Vec<Chunk>, what: Chunk) -> Vec<Chunk> {
    let mut new : Vec<Chunk> = vec![];

    let mut added : bool = false;

    for i in from {
        if i.name == what.name {
            new.push(what.clone());
            added = true;
        } else {
            new.push(i);
        }
    }

    if !added {
        new.push(what.clone());
    }

    new
}

fn find(from: Vec<Chunk>, name: String) -> bool {
    for i in from {
        if i.name == name {
            return true
        }
    }

    false
}

fn get_files(path: String) -> Vec<String> {

    let mut list_files = Command::new("ls");

    list_files.arg("-la");
    if path.len() > 0 {
        list_files.arg(path.clone());
    }

    let files = list_files.output().expect("no");

    let stdout = String::from_utf8(files.stdout).unwrap();

    let mut current = "".to_string();

    let mut gottem = vec![];

    for i in stdout.chars() {
        if i.to_string() != "\n" {
            current += &i.to_string();
        } else {
            gottem.push(current);
            current = "".to_string();
        }
    }

    gottem
}

fn explore_path(from: String, into: String) -> (String, Vec<String>) {
    (from.clone() + into.as_str() + "/", get_files((from + into.as_str() + "/").into()))
}

fn flatten(thick : Vec<Vec<u8>>) -> Vec<u8> {
    let mut new : Vec<u8> = vec![];

    for i in thick {
        for k in i {
            new.push(k);
        }
    }

    new
}

fn compress(wide : Vec<u8>) -> Vec<u8> {
    let mut now : u8 = 0;
    let mut chn : i32 = 0;

    let mut new : Vec<u8> = vec![];

    for i in wide {
        if chn == 0 {
            now = i.into();
        } else {
            now += i << 4;
            new.push(now);
            now = 0;
        }
        chn = 1 - chn;
    }

    new
}

fn expand(from: Vec<(u8, u8, u8)>) -> Vec<u8> {
    let mut new : Vec<u8> = vec![];

    for i in from {
        new.push(i.0);
        new.push(i.1);
        new.push(i.2);
    }

    new
}

fn draw_img(what: Vec<i32>) -> () {
    let mut idx = 0;

    let col = [BLACK, WHITE];
    let mut cid = 0;

    for i in what.clone() {
        for _k in 0..i {
            draw_rectangle((idx as f32 % 8.0) * PIX_SIZE, (idx as f32 / 8.0) as i32 as f32 * PIX_SIZE, PIX_SIZE, PIX_SIZE, col[cid]);

            idx += 1;
        }
        cid += 1;
        cid = cid % 2;
    }
}

fn construct_tic(path: String, from: Vec<Chunk>) -> () {
    println!("{}", path);

    let mut file = fs::OpenOptions::new().create(true).write(true).open(path).expect("No");

    for i in &from {
        let type_id = match i.name.as_str() {
            "Tiles" => 1,
            "Sprites" => 2,
            "Map" => 4,
            "Code" => 5,
            "Flags" => 6,
            "Samples" => 9,
            "Waveform" => 10,
            "Palette" => 12,
            "Music" => 14,
            "Patterns" => 15,
            "Default" => 17,
            "Screen" => 18,
            "Binary" => 19,
            _ => 32,
        } & 0b00011111 ;

        println!("{}", type_id);

        let size = i.data.len() as u16;

        println!("{}", size);

        let size_low : u8 = (size & 0b0000000011111111) as u8;
        let size_high : u8 = ((size & 0b1111111100000000) >> 8) as u8;

        println!("{} {}", size_low, size_high);

        let bank = i.bank >> 5;

        println!("{}", bank);

        let mut chunk_bytes : Vec<u8> = vec![];

        println!("{}", bank + type_id);

        chunk_bytes.push(bank + type_id);
        chunk_bytes.push(size_low);
        chunk_bytes.push(size_high);
        chunk_bytes.push(0);

        for k in i.data.clone() {
            chunk_bytes.push(k);
        }

        let _ = file.write_all(&chunk_bytes);
    }
}

#[macroquad::main("ArTic Editor")]
async fn main() {

    let save_image = [9, 5, 3, 1, 4, 1, 2, 1, 4, 1, 2, 1, 1, 2, 1, 1, 2, 1, 4, 1, 2, 6];

    // macroquad is cool

    let mut show_spr : bool = false;

    let mut black_pal : (u8, u8, u8) = (0, 0, 0);

    let (mut sel_x, mut sel_y) : (f32, f32) = (-16.0 * PIX_SIZE, -16.0 * PIX_SIZE);
    let (mut sel_w, mut sel_h) : (f32, f32) = (SPR_SIDE_LENGTH, SPR_SIDE_LENGTH);

    let mut last_press_l : bool = false;
    let mut last_press_r : bool = false;

    let args : Vec<String> = env::args().collect();

    let mut current_state : &str = "open";

    let mut file_path = "secret.tic".to_string();
    let mut search_path = "".to_string();

    if args.len() > 1 {
        let par : Vec<&str> = args[1].split(".").collect();

        if par[par.len()-1] == "tic" {
            current_state = "read_file";
            file_path = args[1].clone();
        } else {
            search_path = args[1].clone();
        }
        println!("{}", args[1])
    }


    let mut palette : Vec<(u8, u8, u8)> = vec![];
    let mut tiles : Vec<Vec<u8>> = vec![];
    let mut sprites : Vec<Vec<u8>> = vec![];

    let mut gottem = get_files(search_path.clone().into());

    gottem.remove(0);
    gottem.remove(0);

    let mut offset = 0.0;

    let mut primary = 0;
    let mut secondary = 0;

    let mut to_draw : Vec<Vec<u8>> = vec![];

    let mut chunks : Vec<Chunk> = vec![];

    loop {
        let current_press_l = is_mouse_button_down(MouseButton::Left);
        let current_press_r = is_mouse_button_down(MouseButton::Right);

        clear_background(BLACK);

        let (mx, my) : (f32, f32) = mouse_position();

        let draw = match show_spr {
            true => &mut sprites,
            false => &mut tiles,
        };

        match current_state {
            "open" => {
                draw_text("Select a file", 50.0, 50.0, 25.0, WHITE);

                let max = match gottem.len() > 19 {
                    true => 19,
                    false => gottem.len(),
                };

                let mw = mouse_wheel().1;

                for i in 0..max {
                    if max < gottem.len() {
                        if mw > 0.0 && offset > 0.0 {
                            offset -= 0.1;
                        } else if mw < 0.0 && offset + 19.0 < gottem.len() as f32 {
                            offset += 0.1;
                        }
                    }


                    let ypos = 100.0 + (i as f32) * 25.0;

                    let text = gottem[i + offset as usize].clone();

                    let is_dir : bool = text[0..1] == *"d" || text.contains("/");

                    let is_symlink : bool = text.contains("/");

                    let div : Vec<&str> = text.split(" ").collect();

                    let mut name = div[div.len()-1];
                    if name == ".." { name = "Up a Level"; }

                    let par : Vec<&str> = text.split(".").collect();

                    let is_tic = par[par.len()-1] == "tic";
                    let is_img = IMG_EXTENTIONS.contains(&par[par.len()-1]);

                    let txt_size = measure_text(&name, None, 25, 1.0);

                    let is_sel : bool = my >= ypos - txt_size.height && my < ypos;

                    let (sel_col, txt_col) = match (is_dir, is_sel, is_tic, is_img) {
                        (false, true, false, false) => (RED, BLACK),
                        (false, false, false, false) => (BLACK, RED),
                        (false, true, false, true) => (YELLOW, BLACK),
                        (false, false, false, true) => (BLACK, YELLOW),
                        (false, true, true, false) => (GREEN, BLACK),
                        (false, false, true, false) => (BLACK, GREEN),
                        (true, true, _tic,  _img) => (WHITE, BLACK),
                        (true, false,  _tic,  _img) => (BLACK, WHITE),
                        _ => (BLACK, BLACK),
                    };

                    if is_sel && current_press_l && !last_press_l {
                        if is_dir {
                            (search_path, gottem) = explore_path(search_path, div[div.len()-1].into());

                            if is_symlink {
                                search_path = name.to_string().clone();
                                gottem = get_files(name.into());
                            }

                            for _i in 0..2 {
                                if gottem.len() > 0 {
                                    gottem.remove(0);
                                }
                            }

                            offset = 0.0;

                            break
                        } else if is_tic {
                            if search_path.len() > 0 {
                                file_path = (search_path.clone() + "/" + name).clone();
                            } else {
                                file_path = name.to_string().clone();
                            }
                            current_state = "read_file";

                            println!("path : {} name : {}", file_path, name);

                            break
                        }
                    }

                    draw_rectangle(49.0, ypos - txt_size.height - 1.0, txt_size.width + 2.0, txt_size.height + 3.0, sel_col);

                    draw_text(&name, 50.0, ypos, 25.0, txt_col);
                }

                draw_text("Direct Import", 600.0, 100.0, 25.0, GREEN);
                draw_text("Convertion", 600.0, 125.0, 25.0, YELLOW);
                draw_text("Cannot Import", 600.0, 150.0, 25.0, RED);
                draw_text("Directory", 600.0, 175.0, 25.0, WHITE);
            },
            "read_file" => {
                draw_text(&("Reading ".to_owned() + file_path.as_str()), 50.0, 50.0, 25.0, WHITE);
                // create a vec of chunks from a .tic

                chunks = deconstruct_tic(file_path.clone().into());

                // see if the default palette (and waveforms) should be loaded

                let default : bool = find(chunks.clone(), "Default".into());

                println!("{default}");

                if default {
                    chunks = replace(
                        chunks.clone(),
                        Chunk{
                            bank : 0,
                            data : DEFAULT_PALETTE.to_vec().clone(),
                            name : "Palette".to_string(),
                        }
                    );
                }

                // extract tiles, sprites and palette

                let mut col_ind : u8 = 0;
                let mut cur_col : [u8; 3] = [0, 0, 0];

                let mut cur_tile : Vec<u8> = vec![];

                let mut cur_spr : Vec<u8> = vec![];

                for imp in IMPORTANT {
                    let chunk_imp = extract(chunks.clone(), imp.into());

                    let data_imp = chunk_imp.data;
                    let bank_imp = chunk_imp.bank;
                    let type_imp = chunk_imp.name;

                    println!("{} byte long {} chunk in bank {}", data_imp.len(), type_imp, bank_imp);

                    if data_imp.len() > 0 {
                        for k in data_imp {

                            let high : u8 = (k & 0b11110000) >> 4;
                            let low : u8 = k & 0b00001111;

                            match type_imp.as_str() {
                                "Tiles" => {
                                    //print!("{} {} ", low, high);
                                    cur_tile.push(low);
                                    cur_tile.push(high);

                                    if cur_tile.len() == 64 {
                                        tiles.push(cur_tile.clone());
                                        cur_tile.clear()
                                    }
                                },
                                "Sprites" => {
                                    //print!("{} {} ", low, high);
                                    cur_spr.push(low);
                                    cur_spr.push(high);

                                    if cur_spr.len() == 64 {
                                        sprites.push(cur_spr.clone());
                                        cur_spr.clear()
                                    }
                                },
                                _ => {
                                    //print!("{} ", k);
                                    cur_col[col_ind as usize] = k;
                                    col_ind += 1;
                                    if col_ind == 3 {
                                        palette.push((
                                            cur_col[0],
                                            cur_col[1],
                                            cur_col[2]
                                        ));
                                        col_ind = 0;
                                    }
                                },
                            }
                        }
                        //print!("\n");
                    }
                }

                // il love \x1B[38;2;{};{};{}m{}\x1B[0m

                println!("\nPalette");
                for i in &palette {
                    println!("\x1B[38;2;{};{};{}m#{:x}{:x}{:x}\x1B[0m", i.0, i.1, i.2, i.0, i.1, i.2);
                }
                print!("\n");

                println!("Tiles");
                for i in &tiles {
                    for y in 0..8 {
                        for k in 0..8 {
                            let col = palette[i[k + y*8] as usize];

                            print!("\x1B[38;2;{};{};{}m{:0>2}\x1B[0m", col.0, col.1, col.2, i[k + y*8]);
                        }
                        print!("\n");
                    }
                    print!("\n")
                }

                println!("Sprites");
                for i in &sprites {
                    for y in 0..8 {
                        for k in 0..8 {
                            let col = palette[i[k + y*8] as usize];

                            print!("\x1B[38;2;{};{};{}m{:0>2}\x1B[0m", col.0, col.1, col.2, i[k + y*8]);
                        }
                        print!("\n");
                    }
                    print!("\n")
                }

                black_pal = palette[0];

                current_state = "main";
            },
            "main" => {
                draw_rectangle(OFF_X , OFF_Y , ALL_SIDE_LENGTH , ALL_SIDE_LENGTH, color_u8!(255, 255, 255, 125));
                draw_rectangle(OFF_X , OFF_Y , ALL_SIDE_LENGTH, ALL_SIDE_LENGTH, color_u8!(black_pal.0, black_pal.1, black_pal.2, 125));

                if is_key_pressed(KeyCode::Tab) {
                    show_spr = !show_spr;
                }

                for id in 0..draw.len() {
                    let i = &draw[id];

                    for y in 0..8 {
                        for k in 0..8 {
                            let col = palette[i[k + y*8] as usize];
                            let my_col = color_u8!(col.0, col.1, col.2, 255);

                            let sx = ((id as f32)%16.0)*PIX_SIZE*8.0;
                            let sy = ((id as f32)/16.0).floor()*PIX_SIZE*8.0;

                            let px = OFF_X + (k as f32)*PIX_SIZE + sx;
                            let py = OFF_Y + (y as f32)*PIX_SIZE + sy;

                            draw_rectangle(px, py, PIX_SIZE, PIX_SIZE, my_col);
                        }
                    }
                }

                for x in 0..16 {
                    for y in 0..16 {

                        let px = OFF_X + (x as f32) * SPR_SIDE_LENGTH;
                        let py = OFF_Y + (y as f32) * SPR_SIDE_LENGTH;

                        if px >= mx.floor() - SPR_SIDE_LENGTH
                        && py >= my.floor() - SPR_SIDE_LENGTH
                        && px <= mx.floor()
                        && py <= my.floor() {
                            if current_press_l {
                                if last_press_l {
                                    (sel_w, sel_h) = (px - sel_x + SPR_SIDE_LENGTH, py - sel_y + SPR_SIDE_LENGTH);
                                } else {
                                    (sel_x, sel_y) = (px, py);
                                    (sel_w, sel_h) = (SPR_SIDE_LENGTH, SPR_SIDE_LENGTH);
                                }
                            }
                        }
                    }
                }

                draw_rectangle_lines(sel_x - SELECTION_THICK / 2.0, sel_y - SELECTION_THICK / 2.0, sel_w + SELECTION_THICK, sel_h + SELECTION_THICK, SELECTION_THICK, WHITE);

                if !current_press_l && sel_x > 0.0 && sel_y > 0.0 {
                    current_state = "edit";
                }

                to_draw.clear();

                for x in 0..16 {
                    for y in 0..16 {
                        let px = OFF_X + (x as f32) * SPR_SIDE_LENGTH;
                        let py = OFF_Y + (y as f32) * SPR_SIDE_LENGTH;

                        if px >= sel_x && py >= sel_y
                        && px < sel_x + sel_w && py < sel_y + sel_h {
                            if x + y * 16 < draw.len() {
                                to_draw.push(draw[(x + y * 16) as usize].clone());
                            } else {
                                to_draw.push(EMPTY_SPR.to_vec().clone());
                            }
                        }
                    }
                }

                //draw_rectangle(0.0, 0.0, 50.0, 50.0, WHITE);
                /*draw_texture_ex(&save, 0.0, 0.0, WHITE, DrawTextureParams {
                    dest_size : Some(Vec2 {
                        x : 50.0,
                        y : 50.0,
                        }),
                    rotation : 0.0,
                    flip_x : false,
                    flip_y : false,
                    pivot : None,
                    source : None,
                });*/

                draw_img(save_image.to_vec());

                if mx < PIX_SIZE * 8.0 && my < PIX_SIZE * 8.0 && current_press_l && !last_press_l {
                    current_state = "saving";
                }
            },
            "edit" => {

                draw_rectangle(PALETTE_SIZE - SELECTION_THICK, SCREEN_HEIGHT / 2.0 - 8.0 * PALETTE_SIZE - SELECTION_THICK, PALETTE_SIZE + SELECTION_THICK * 2.0, PALETTE_SIZE * 16.0 + SELECTION_THICK * 2.0, WHITE);
                draw_rectangle(PALETTE_SIZE - SELECTION_THICK, SCREEN_HEIGHT / 2.0 + 9.0 * PALETTE_SIZE - SELECTION_THICK, PALETTE_SIZE + SELECTION_THICK * 2.0, PALETTE_SIZE + SELECTION_THICK * 2.0, WHITE);
                draw_rectangle(PALETTE_SIZE * 1.5 - SELECTION_THICK, SCREEN_HEIGHT / 2.0 + 9.5 * PALETTE_SIZE - SELECTION_THICK, PALETTE_SIZE + SELECTION_THICK * 2.0, PALETTE_SIZE + SELECTION_THICK * 2.0, WHITE);

                for c in 0..palette.len() {
                    let col = palette[c];

                    let cy = SCREEN_HEIGHT / 2.0 - 8.0 * PALETTE_SIZE + (c as f32) * PALETTE_SIZE;

                    let my_col = color_u8!(col.0, col.1, col.2, 255);

                    if mx < PALETTE_SIZE * 2.0 + SELECTION_THICK
                    && my >= cy && my < cy + PALETTE_SIZE {
                        if current_press_l && !last_press_l {
                            primary = c;
                        } else if current_press_r && !last_press_r {
                            secondary = c;
                        }
                    }

                    draw_rectangle(PALETTE_SIZE, cy, PALETTE_SIZE, PALETTE_SIZE, my_col);
                }

                let col = palette[secondary];

                draw_rectangle(PALETTE_SIZE, SCREEN_HEIGHT / 2.0 + 9.0 * PALETTE_SIZE, PALETTE_SIZE, PALETTE_SIZE, color_u8!(col.0, col.1, col.2, 255));

                let col = palette[primary];

                draw_rectangle(PALETTE_SIZE * 1.5, SCREEN_HEIGHT / 2.0 + 9.5 * PALETTE_SIZE, PALETTE_SIZE, PALETTE_SIZE, color_u8!(col.0, col.1, col.2, 255));

                let mult : f32;

                if (SCREEN_HEIGHT - 6.0 * PALETTE_SIZE) / sel_h < (SCREEN_WIDTH - 6.0 * PALETTE_SIZE) / sel_w {
                    mult = (SCREEN_HEIGHT - 6.0 * PALETTE_SIZE) / sel_h;
                } else {
                    mult = (SCREEN_WIDTH - 6.0 * PALETTE_SIZE) / sel_w;
                }

                let (mut drax, mut dray) : (usize, usize) = (0, 0);

                let mut hover : bool = false;

                for idx in 0..to_draw.len() {
                    let l = &mut to_draw[idx as usize];

                    for id in 0..l.len() {
                        let i = &mut l[id as usize];

                        let col = palette[*i as usize];
                        let my_col = color_u8!(col.0, col.1, col.2, 255);

                        let x = (id as f32)%8.0 * PIX_SIZE * mult;
                        let y = ((id as f32)/8.0).floor() * PIX_SIZE * mult;

                        let ofx = ((idx as f32)/(sel_h / SPR_SIDE_LENGTH)).floor() * SPR_SIDE_LENGTH * mult;
                        let ofy = (idx as f32)%(sel_h / SPR_SIDE_LENGTH) * SPR_SIDE_LENGTH * mult;

                        let bx = (SCREEN_WIDTH - sel_w * mult) / 2.0;
                        let by = (SCREEN_HEIGHT - sel_h * mult) / 2.0;

                        draw_rectangle(x + ofx + bx, y + ofy + by, PIX_SIZE * mult, PIX_SIZE * mult, my_col);

                        if mx >= x + ofx + bx
                        && mx < x + ofx + bx + PIX_SIZE * mult
                        && my >= y + ofy + by
                        && my < y + ofy + by + PIX_SIZE * mult {
                            hover = true;

                            if current_press_l || current_press_r {
                                drax = idx;
                                dray = id;
                            }

                            draw_rectangle_lines(x + ofx + bx, y + ofy + by, PIX_SIZE * mult, PIX_SIZE * mult, SELECTION_THICK, BLACK)
                        }
                    }
                }

                if mx > PALETTE_SIZE * 2.0 + SELECTION_THICK && hover {
                    if current_press_l {
                        to_draw[drax][dray] = primary as u8;
                    } else if current_press_r {
                        to_draw[drax][dray] = secondary as u8;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    // save modified pixels

                    let zx = (sel_x / PIX_SIZE - 61.0) / 8.0;
                    let zy = (sel_y / PIX_SIZE - 11.0) / 8.0;

                    let zid = zx + zy * 16.0;


                    for x in 0..to_draw.len() {
                        let i = to_draw[x].clone();

                        let ox = (x as f32 / (sel_h / SPR_SIDE_LENGTH)) as i32 as f32;

                        for y in 0..i.len() {

                            let oy = x as f32 % (sel_h / SPR_SIDE_LENGTH);

                            let zo = ox + oy * 16.0;

                            if draw.len() as f32 > zid + zo {
                                draw[(zid + zo) as usize][y] = to_draw[x][y];
                            } else {
                                while (draw.len() as f32) < zid + zo {
                                    draw.push(EMPTY_SPR.to_vec().clone())
                                }
                                draw.push(to_draw[x].clone());
                                break
                            }
                        }
                    }

                    current_state = "main";
                    (sel_x, sel_y) = (-16.0 * PIX_SIZE, -16.0 * PIX_SIZE);
                    (sel_w, sel_h) = (SPR_SIDE_LENGTH, SPR_SIDE_LENGTH);
                }
            },
            "saving" => {

                let comp_tiles = compress(flatten(tiles.clone()));
                let comp_sprites = compress(flatten(sprites.clone()));
                let exp_palette = expand(palette.clone());

                chunks = replace(chunks, Chunk { name : "Tiles".into(), bank : 0, data : comp_tiles});
                chunks = replace(chunks, Chunk { name : "Sprites".into(), bank : 0, data : comp_sprites});
                chunks = replace(chunks, Chunk { name : "Palette".into(), bank : 0, data : exp_palette});

                construct_tic(file_path.into(), chunks);

                current_state = "open";

                exit(0x0100);
            },
            _ => {},
        }

        last_press_l = current_press_l;
        last_press_r = current_press_r;

        next_frame().await
    }
}
