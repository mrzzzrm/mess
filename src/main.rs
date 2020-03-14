fn main() {
    let mut board = mess::pile::Board::create_king_rooks();
    mess::pile::play(&mut board);
}
