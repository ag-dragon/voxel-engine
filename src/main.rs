use voxel_engine::run;

fn main() {
    futures::executor::block_on(run());
}
