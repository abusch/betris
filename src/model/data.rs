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
            Pos::new(1, -2),
            Pos::new(1, -1),
            Pos::new(1, 0),
            Pos::new(1, 1),
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
            Pos::new(0, -2),
            Pos::new(0, -1),
            Pos::new(0, 0),
            Pos::new(0, 1),
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

/// The O-Tetrimino looks the same in every facing, so it only ever uses Rotation Point 1
pub const O_ROTATION_POINTS: [Pos; 5] = [Pos::ZERO; 5];

/// The Super Rotation System's Rotation Points for the J, L, S, T and Z-Tetriminos, as offsets
/// from the piece's position. Indexed by start facing, then by `Rotation` (clockwise, then
/// counterclockwise).
pub const JLSTZ_ROTATION_POINTS: [[[Pos; 5]; 2]; 4] = [
    // North -> East, North -> West
    [
        points([(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]),
        points([(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)]),
    ],
    // East -> South, East -> North
    [
        points([(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]),
        points([(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]),
    ],
    // South -> West, South -> East
    [
        points([(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)]),
        points([(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)]),
    ],
    // West -> North, West -> South
    [
        points([(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)]),
        points([(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)]),
    ],
];

/// The Super Rotation System's Rotation Points for the I-Tetrimino, indexed like
/// `JLSTZ_ROTATION_POINTS`
pub const I_ROTATION_POINTS: [[[Pos; 5]; 2]; 4] = [
    // North -> East, North -> West
    [
        points([(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)]),
        points([(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)]),
    ],
    // East -> South, East -> North
    [
        points([(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)]),
        points([(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)]),
    ],
    // South -> West, South -> East
    [
        points([(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)]),
        points([(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)]),
    ],
    // West -> North, West -> South
    [
        points([(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)]),
        points([(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)]),
    ],
];

const fn points(offsets: [(i8, i8); 5]) -> [Pos; 5] {
    let mut points = [Pos::ZERO; 5];
    let mut i = 0;
    while i < offsets.len() {
        points[i] = Pos::new(offsets[i].0, offsets[i].1);
        i += 1;
    }
    points
}
