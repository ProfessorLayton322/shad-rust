#![forbid(unsafe_code)]

pub struct TarFile<'a> {
    pub header: TarHeader<'a>,
    pub data: &'a [u8],
}

pub struct TarHeader<'a> {
    pub name: &'a [u8],
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub data_size: usize,
}

const BLOCK_SIZE: usize = 512;

pub fn cut_null(slice: &[u8]) -> &[u8] {
    let null_terminator = slice.iter().position(|byte| *byte == 0).unwrap();
    &slice[0..null_terminator]
}

pub fn read_slice(mut slice: &[u8]) -> u32 {
    slice = cut_null(slice);
    u32::from_str_radix(std::str::from_utf8(slice).unwrap(), 8).unwrap()
}

pub fn parse_tar(tar: &[u8]) -> Vec<TarFile> {
    let mut answer = Vec::<TarFile>::new();
    if tar.len() % BLOCK_SIZE > 0 {
        panic!("Something wrong with data length");
    }
    let mut block_num = tar.len() / BLOCK_SIZE;
    if block_num < 2 {
        panic!("Data too short");
    }
    block_num -= 2;
    let end_blocks = &tar[block_num * BLOCK_SIZE..tar.len()];
    if !end_blocks.iter().all(|byte| *byte == 0) {
        panic!("No two null blocks in the end");
    }
    let mut blocks_behind: usize = 0;
    while blocks_behind < block_num {
        let header_block: &[u8] =
            &tar[blocks_behind * BLOCK_SIZE..(blocks_behind + 1) * BLOCK_SIZE];
        blocks_behind += 1;
        let tar_header = TarHeader {
            name: cut_null(&header_block[0..100]),
            mode: read_slice(&header_block[100..108]),
            uid: read_slice(&header_block[108..116]),
            gid: read_slice(&header_block[116..124]),
            data_size: read_slice(&header_block[124..136]) as usize,
        };
        let data_blocks = (tar_header.data_size + BLOCK_SIZE - 1) / BLOCK_SIZE;
        if blocks_behind + data_blocks > block_num {
            panic!("Not enough blocks");
        }
        let data_begin = blocks_behind * BLOCK_SIZE;
        let tar_file = TarFile {
            data: &tar[data_begin..data_begin + tar_header.data_size],
            header: tar_header,
        };
        answer.push(tar_file);
        blocks_behind += data_blocks;
    }
    answer
}
