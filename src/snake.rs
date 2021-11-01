use core::iter::Cycle;
use core::str::Chars;
use rand::prelude::*;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{Stdout, Write};
use std::ops;

use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::size;

use std::thread::sleep;
use std::time::Duration;

// blame mahasvan for this "secret" snake option

#[derive(Debug, PartialEq, Copy, Clone)]
struct CoordinateVector(pub i32, pub i32);
impl ops::Add for CoordinateVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CoordinateVector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

struct Snake {
    seg: VecDeque<CoordinateVector>,
    direction: CoordinateVector,
}

fn travel(snake: &mut Snake, grow: bool) -> CoordinateVector {
    let &old_head = snake.seg.back().unwrap();
    if grow {
        let &old_tail = snake.seg.front().unwrap();
        for _ in 0..5 {
            snake.seg.push_front(old_tail);
        }
    } else {
        snake.seg.pop_front().unwrap();
    }
    let new_head = old_head + snake.direction;
    snake.seg.push_back(old_head + snake.direction);
    new_head
}

fn head_touching_object(snake: &Snake, object: CoordinateVector) -> bool {
    *snake.seg.back().unwrap() == object
}

fn head_touching_snake(snake: &Snake, other: &Snake) -> bool {
    let &head = snake.seg.back().unwrap();
    // Find the position of first snake segment which is equal to the head
    let position = match other.seg.iter().position(|&coord| coord == head) {
        Some(p) => p,
        None => 100,
    };
    // Return true if the found position is not the head.
    position < other.seg.len() - 1
}

fn head_out_of_bounds(snake: &Snake, bounds: CoordinateVector) -> bool {
    let &head = snake.seg.back().unwrap();
    head.0 > bounds.0 || head.1 > bounds.1 || head.0 < 1 || head.1 < 1
}

pub fn snake(stdout: &mut Stdout) -> core::result::Result<(), Box<dyn Error>> {
    let mut masc = "BLAME_MAHASVAN_FOR_THIS_".chars().cycle();
    let mut apple = "113322446655".chars().cycle();
    let mut stripe = "13".chars().cycle();
    let mut score = 0;

    write!(stdout, "\x1b[2J")?;

    let (y1, x1) = size()?;
    let x = i32::from(x1);
    let y = i32::from(y1);

    let mut rng = thread_rng();
    let board_bounds = CoordinateVector(y + 1, x + 1);
    let mut snake = Snake {
        seg: VecDeque::from(vec![CoordinateVector(y / 2, x / 2)]),
        direction: CoordinateVector(1, 0),
    };
    let mut baddy = Snake {
        seg: VecDeque::from(vec![CoordinateVector(
            rng.gen_range(1..board_bounds.0),
            rng.gen_range(1..board_bounds.1),
        )]),
        direction: CoordinateVector(0, 1),
    };
    let &tail = baddy.seg.front().unwrap();
    for _ in 0..8 {
        baddy.seg.push_front(tail);
    }
    let mut food = get_new_food_position(&snake, board_bounds, &mut rng);

    travel(&mut snake, true);

    loop {
        let eating_food = head_touching_object(&snake, food);
        if eating_food {
            score += 1;
            food = get_new_food_position(&snake, board_bounds, &mut rng);
        }
        travel(&mut snake, eating_food);
        let t = rng.gen_range(1..100);
        if t > 95 {
            turn_right(&mut baddy);
        } else if t < 5 {
            turn_left(&mut baddy);
        }
        travel(&mut baddy, false);
        if head_out_of_bounds(&baddy, board_bounds) {
            baddy.seg.pop_back().unwrap();
            let &tail = baddy.seg.front().unwrap();
            baddy.seg.push_front(tail);
            turn_right(&mut baddy);
        };
        display(
            stdout,
            &snake,
            &baddy,
            food,
            &mut masc,
            &mut apple,
            &mut stripe,
            score,
        );
        if head_touching_snake(&snake, &snake)
            || head_out_of_bounds(&snake, board_bounds)
            || head_touching_snake(&baddy, &snake)
        {
            break;
        }
        stdout.flush().unwrap();
        if poll(Duration::from_millis(100)).ok().unwrap() {
            if let Event::Key(ke) = read().ok().unwrap() {
                match ke.code {
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
                        if snake.direction.1 != 0 {
                            snake.direction = CoordinateVector(-1, 0);
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
                        if snake.direction.1 != 0 {
                            snake.direction = CoordinateVector(1, 0);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        if snake.direction.0 != 0 {
                            snake.direction = CoordinateVector(0, -1);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        if snake.direction.0 != 0 {
                            snake.direction = CoordinateVector(0, 1);
                        }
                    }
                    KeyCode::Esc => break,
                    _ => (),
                }
            }
        };
    }
    for segment in snake.seg.iter() {
        write!(
            stdout,
            "\x1B[{};{}H\x1B[31;7m{}\x1B[0m",
            segment.1, segment.0, 'X'
        )
        .unwrap();
        stdout.flush().unwrap();
        sleep(Duration::from_millis(5));
    }
    Ok(())
}

fn get_new_food_position(
    snake: &Snake,
    bounds: CoordinateVector,
    rng: &mut ThreadRng,
) -> CoordinateVector {
    let new_position = CoordinateVector(rng.gen_range(1..bounds.0), rng.gen_range(1..bounds.1));
    match snake.seg.contains(&new_position) {
        true => get_new_food_position(snake, bounds, rng),
        false => new_position,
    }
}

fn turn_left(snake: &mut Snake) {
    let mut a = snake.direction.0;
    let mut b = snake.direction.1;
    if a != 0 {
        b = -a;
        a = 0;
    } else {
        a = b;
        b = 0;
    }
    snake.direction = CoordinateVector(a, b);
}

fn turn_right(snake: &mut Snake) {
    let mut a = snake.direction.0;
    let mut b = snake.direction.1;
    if a != 0 {
        b = a;
        a = 0;
    } else {
        a = -b;
        b = 0;
    }
    snake.direction = CoordinateVector(a, b);
}

fn display(
    stdout: &mut Stdout,
    snake: &Snake,
    baddy: &Snake,
    food: CoordinateVector,
    snk: &mut Cycle<Chars>,
    apple: &mut Cycle<Chars>,
    stripe: &mut Cycle<Chars>,
    score: i32,
) {
    write!(
        stdout,
        "\x1B[{};{}H\x1B[3{}m{}\x1B[0m",
        food.1,
        food.0,
        apple.next().unwrap(),
        ''
    )
    .unwrap();
    let segment = snake.seg.back().unwrap();
    write!(
        stdout,
        "\x1B[{};{}H\x1B[42;37m{}\x1B[0m",
        segment.1,
        segment.0,
        snk.next().unwrap()
    )
    .unwrap();
    let segment = snake.seg.front().unwrap();
    write!(stdout, "\x1B[{};{}H{}", segment.1, segment.0, ' ').unwrap();
    let segment = baddy.seg.back().unwrap();
    write!(
        stdout,
        "\x1B[{};{}H\x1B[3{};7m{}\x1B[0m",
        segment.1,
        segment.0,
        stripe.next().unwrap(),
        '/'
    )
    .unwrap();
    let segment = baddy.seg.front().unwrap();
    write!(stdout, "\x1B[{};{}H{}", segment.1, segment.0, ' ').unwrap();
    write!(stdout, "\x1B[1;1H{}", score).unwrap();
}
