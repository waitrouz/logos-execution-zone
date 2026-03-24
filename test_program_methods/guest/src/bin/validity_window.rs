use nssa_core::program::{
    AccountPostState, BlockId, ProgramInput, ProgramOutput, Timestamp, read_nssa_inputs,
};

type Instruction = (
    Option<BlockId>,
    Option<BlockId>,
    Option<Timestamp>,
    Option<Timestamp>,
);

fn main() {
    let (
        ProgramInput {
            pre_states,
            instruction: validity_window,
        },
        instruction_words,
    ) = read_nssa_inputs::<Instruction>();

    let Ok([pre]) = <[_; 1]>::try_from(pre_states) else {
        return;
    };

    let post = pre.account.clone();

    ProgramOutput::new(
        instruction_words,
        vec![pre],
        vec![AccountPostState::new(post)],
    )
    .try_with_validity_window(validity_window)
    .unwrap()
    .write();
}
