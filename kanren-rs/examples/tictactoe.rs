
use kanren_rs::{display::*, *};

goal!(
    fn step(current_board: Var, player: Var, next_board: Var) -> Goal {
        fresh(move |a, b, c, d, e, f, g, h| {
            cond([
                vec![eq(current_board, list!(a,b,c,d,e,f,g,h,"-")), eq(next_board, list!(a,b,c,d,e,f,g,h,player))],
                vec![eq(current_board, list!(a,b,c,d,e,f,g,"-",h)), eq(next_board, list!(a,b,c,d,e,f,g,player,h))],
                vec![eq(current_board, list!(a,b,c,d,e,f,"-",g,h)), eq(next_board, list!(a,b,c,d,e,f,player,g,h))],
                vec![eq(current_board, list!(a,b,c,d,e,"-",f,g,h)), eq(next_board, list!(a,b,c,d,e,player,f,g,h))],
            ])
        })
    }
);


fn main() {
    let states = run(10, |board| fresh(move |board0, board1, board2, player1, player2| {
        all([
            eq(board0, list!("-","-","-","-","-","-","-","-","-")),
            eq(player1, "X"),
            eq(player2, "O"),
            step( board0, player1, board1),
            step( board1, player2, board2),
            step( board2, player1, board),
        ])
    }));

    for state in states {
        print!("{}\n", AsScheme(state));
    }
}