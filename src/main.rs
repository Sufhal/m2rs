use game::run;

fn main() {
    pollster::block_on(run());
}
