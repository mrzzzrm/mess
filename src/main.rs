use mess::*;

fn main() {
    let mut board = board::Board::create_king_rooks();
    play(&mut board);
}
