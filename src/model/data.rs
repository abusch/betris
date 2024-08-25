use super::Pos;

pub const OFFSETS: [[[Pos; 4]; 4]; 7] = [
    // O-Tetrimino
    [
        // North
        [
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, 1),
            Pos::new(0, 1),
        ],
        // East
        [
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, 1),
            Pos::new(0, 1),
        ],
        // South
        [
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, 1),
            Pos::new(0, 1),
        ],
        // West
        [
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, 1),
            Pos::new(0, 1),
        ],
    ],
    // I-Tetrimino
    [
        // North
        [
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(2, 0),
        ],
        // East
        [
            Pos::new(1, -1),
            Pos::new(1, 0),
            Pos::new(1, 1),
            Pos::new(1, 2),
        ],
        // South
        [
            Pos::new(-1, -1),
            Pos::new(0, -1),
            Pos::new(1, -1),
            Pos::new(2, -1),
        ],
        // West
        [
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(0, 1),
            Pos::new(0, 2),
        ],
    ],
    // T-Tetrimino
    [
        // North
        [
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(0, 1),
        ],
        // East
        [
            Pos::new(0, 1),
            Pos::new(0, 0),
            Pos::new(0, -1),
            Pos::new(1, 0),
        ],
        // South
        [
            Pos::new(1, 0),
            Pos::new(0, 0),
            Pos::new(-1, 0),
            Pos::new(0, -1),
        ],
        // West
        [
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(0, 1),
            Pos::new(-1, 0),
        ],
    ],
    // L-Tetrimino
    [
        // North
        [
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, 1),
        ],
        // East
        [
            Pos::new(0, 1),
            Pos::new(0, 0),
            Pos::new(0, -1),
            Pos::new(1, -1),
        ],
        // South
        [
            Pos::new(1, 0),
            Pos::new(0, 0),
            Pos::new(-1, 0),
            Pos::new(-1, -1),
        ],
        // West
        [
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(0, 1),
            Pos::new(-1, 1),
        ],
    ],
    // J-Tetrimino
    [
        // North
        [
            Pos::new(-1, 1),
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(1, 0),
        ],
        // East
        [
            Pos::new(1, 1),
            Pos::new(0, 1),
            Pos::new(0, 0),
            Pos::new(0, -1),
        ],
        // South
        [
            Pos::new(1, -1),
            Pos::new(1, 0),
            Pos::new(0, 0),
            Pos::new(-1, 0),
        ],
        // West
        [
            Pos::new(-1, -1),
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(0, 1),
        ],
    ],
    // S-Tetrimino
    [
        // North
        [
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(0, 1),
            Pos::new(1, 1),
        ],
        // East
        [
            Pos::new(0, 1),
            Pos::new(0, 0),
            Pos::new(1, 0),
            Pos::new(1, -1),
        ],
        // South
        [
            Pos::new(1, 0),
            Pos::new(0, 0),
            Pos::new(0, -1),
            Pos::new(-1, -1),
        ],
        // West
        [
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(-1, 0),
            Pos::new(-1, 1),
        ],
    ],
    // Z-Tetrimino
    [
        // North
        [
            Pos::new(-1, 1),
            Pos::new(0, 1),
            Pos::new(0, 0),
            Pos::new(1, 0),
        ],
        // East
        [
            Pos::new(1, 1),
            Pos::new(1, 0),
            Pos::new(0, 0),
            Pos::new(0, -1),
        ],
        // South
        [
            Pos::new(1, -1),
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(-1, 0),
        ],
        // West
        [
            Pos::new(-1, -1),
            Pos::new(-1, 0),
            Pos::new(0, 0),
            Pos::new(0, 1),
        ],
    ],
];
