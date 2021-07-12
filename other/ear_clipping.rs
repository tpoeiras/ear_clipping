use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::iter;
use std::ops;

#[derive(Clone, Copy, Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone)]
struct Point {
    label: usize,
    color: Option<Color>,
    x: f64,
    y: f64,
}

fn vertical_intersects(point: &Point, (p1, p2): (&Point, &Point)) -> bool {
    let (p1, p2) = if p1.x > p2.x { (p2, p1) } else { (p1, p2) };

    point.x > p1.x && point.x <= p2.x && turn(p1, p2, point) != Turn::Right
}

impl ops::Sub for &Point {
    type Output = Point;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            label: 0,
            color: None,
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Eq, PartialEq)]
enum Turn {
    Right,
    Left,
    None,
}

fn turn(a: &Point, b: &Point, c: &Point) -> Turn {
    let v1 = b - a;
    let v2 = c - b;

    match (v1.x * v2.y - v2.x * v1.y).partial_cmp(&0.0).unwrap() {
        Ordering::Less => Turn::Right,
        Ordering::Equal => Turn::None,
        Ordering::Greater => Turn::Left,
    }
}

struct Polygon {
    points: Vec<Point>,
}

impl Polygon {
    fn segments(&self) -> impl Iterator<Item = (&Point, &Point)> {
        self.points
            .windows(2)
            .map(|ps| (&ps[0], &ps[1]))
            .chain(iter::once((
                self.points.last().unwrap(),
                self.points.first().unwrap(),
            )))
    }

    fn contains(&self, p: &Point) -> bool {
        self.segments()
            .filter(|&segment| vertical_intersects(p, segment))
            .count()
            % 2
            == 1
    }

    fn ear_clipping(&self) -> Triangulation {
        let idx = (0..self.points.len())
            .into_iter()
            .map(|index| EarPoint {
                is_ear: false,
                index,
            })
            .collect::<Vec<EarPoint>>();
        let mut remaining_points = EarPointsList(idx);

        for i in 0..remaining_points.0.len() {
            let i = i as isize;
            remaining_points[i].is_ear = self.is_ear(&remaining_points, i);
        }

        let mut triangulation = Triangulation::new();
        println!();
        println!(" Ear-Clipping");

        loop {
            let next_index = remaining_points.0.iter().position(|p| p.is_ear).unwrap();

            let next_index_i = next_index as isize;

            triangulation.add(
                (
                    remaining_points[next_index_i - 1].index,
                    remaining_points[next_index_i + 1].index,
                ),
                remaining_points[next_index_i].index,
            );
            println!(
                " -> adicionou diagonal {}-{} à triangulação",
                self.points[remaining_points[next_index_i - 1].index].label,
                self.points[remaining_points[next_index_i + 1].index].label
            );

            remaining_points.0.remove(next_index);

            remaining_points[next_index_i - 1].is_ear =
                self.is_ear(&remaining_points, next_index_i - 1);
            remaining_points[next_index_i].is_ear = self.is_ear(&remaining_points, next_index_i);

            if remaining_points.0.len() <= 3 {
                triangulation.add_last((
                    remaining_points[next_index_i - 1].index,
                    remaining_points[next_index_i + 1].index,
                ));

                break;
            }
        }

        triangulation
    }

    fn is_ear(&self, remaining_points: &EarPointsList, idx: isize) -> bool {
        let previous = &self.points[remaining_points[idx - 1].index];
        let point = &self.points[remaining_points[idx].index];
        let next = &self.points[remaining_points[idx + 1].index];

        turn(previous, point, next) == Turn::Left && {
            let triangle = Polygon {
                points: vec![previous.clone(), point.clone(), next.clone()],
            };

            (0..(idx - 1))
                .into_iter()
                .chain(((idx + 2)..remaining_points.0.len() as isize).into_iter())
                .all(|idx| !triangle.contains(&self.points[remaining_points[idx].index]))
        }
    }

    fn colorize(&mut self, triangulation: Triangulation) {
        let (diagonal, vals) = triangulation.diagonals.iter().next().unwrap();

        println!();
        println!(" Coloração");

        self.points[diagonal.0].color = Some(Color::Red);
        println!(
            " -> coloriu ponto {} com a cor {:?}",
            self.points[diagonal.0].label,
            Color::Red
        );

        self.points[diagonal.1].color = Some(Color::Green);
        println!(
            " -> coloriu ponto {} com a cor {:?}",
            self.points[diagonal.1].label,
            Color::Green
        );

        self.points[vals[0]].color = Some(Color::Blue);
        println!(
            " -> coloriu ponto {} com a cor {:?}",
            self.points[vals[0]].label,
            Color::Blue
        );

        let mut stack = vec![
            (diagonal.0, vals[0], diagonal.1),
            (diagonal.1, vals[0], diagonal.0),
            (diagonal.0, diagonal.1, vals[0]),
        ];
        while let Some((d0, d1, other)) = stack.pop() {
            let (d0, d1) = if d0 > d1 { (d1, d0) } else { (d0, d1) };

            if let Some(vals) = triangulation.diagonals.get(&(d0, d1)) {
                let new = vals.iter().find(|&&v| v != other).unwrap();
                self.points[*new].color = self.points[other].color;
                println!(
                    " -> coloriu ponto {} com a cor {:?}",
                    self.points[*new].label,
                    self.points[*new].color.unwrap()
                );

                stack.push((d0, *new, d1));
                stack.push((d1, *new, d0));
            }
        }
    }

    fn print(&self, print_color: bool) {
        for point in self.points.iter() {
            print!("Ponto {}: ({}, {})", point.label, point.x, point.y);
            if print_color {
                if let Some(color) = point.color {
                    print!(" => {:?}", color);
                }
            }
            println!()
        }
    }
}

struct Triangulation {
    diagonals: HashMap<(usize, usize), Vec<usize>>,
}

impl Triangulation {
    fn new() -> Triangulation {
        Triangulation {
            diagonals: HashMap::new(),
        }
    }

    fn add(&mut self, mut diagonal: (usize, usize), point: usize) {
        if diagonal.0 > diagonal.1 {
            diagonal = (diagonal.1, diagonal.0);
        }

        let entry = self
            .diagonals
            .entry(diagonal)
            .or_insert_with(|| Vec::with_capacity(2));
        entry.push(point);

        let entry0 = if diagonal.0 > point {
            (point, diagonal.0)
        } else {
            (diagonal.0, point)
        };

        self.diagonals
            .entry(entry0)
            .and_modify(|e| e.push(diagonal.1));

        let entry1 = if diagonal.1 > point {
            (point, diagonal.1)
        } else {
            (diagonal.1, point)
        };

        self.diagonals
            .entry(entry1)
            .and_modify(|e| e.push(diagonal.0));
    }

    fn add_last(&mut self, diagonal: (usize, usize)) {
        for (_, v) in self
            .diagonals
            .iter_mut()
            .filter(|((x, y), v)| (*x == diagonal.0 || *y == diagonal.0) && v.len() == 1)
        {
            v.push(diagonal.1);
        }

        for (_, v) in self
            .diagonals
            .iter_mut()
            .filter(|((x, y), v)| (*x == diagonal.1 || *y == diagonal.1) && v.len() == 1)
        {
            v.push(diagonal.0)
        }
    }
}

struct EarPoint {
    is_ear: bool,
    index: usize,
}

struct EarPointsList(Vec<EarPoint>);

impl IntoIterator for EarPointsList {
    type Item = EarPoint;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> ops::Index<isize> for EarPointsList {
    type Output = EarPoint;

    fn index(&self, index: isize) -> &Self::Output {
        if index < 0 {
            return &self.0[(index + self.0.len() as isize) as usize];
        }

        &self.0[(index as usize % self.0.len()) as usize]
    }
}

impl<'a> ops::IndexMut<isize> for EarPointsList {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let index = if index < 0 {
            (index + self.0.len() as isize) as usize
        } else {
            index as usize % self.0.len()
        };

        &mut self.0[index as usize]
    }
}

fn read_point(label: usize) -> Point {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("failed to read input");

    let mut vals = buffer.split_whitespace().map(|v| v.parse().unwrap());

    Point {
        label,
        color: None,
        x: vals.next().unwrap(),
        y: vals.next().unwrap(),
    }
}

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("failed to read input");

    let num = buffer.trim().parse().unwrap();

    let points = (1..=num).map(read_point).collect();
    let mut polygon = Polygon { points };

    println!();
    println!(" Entrada");
    polygon.print(false);

    let triangulation = polygon.ear_clipping();
    polygon.colorize(triangulation);

    println!();
    println!(" Final");
    polygon.print(true);
}
