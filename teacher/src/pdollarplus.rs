use std;
use std::f64::consts::PI;
//use std::time::{Duration, Instant};
/*
$ P + Point-Cloud识别器是一款2-D手势识别器，专为基于手势的用户界面进行快速原型设计，尤其适用于视力不佳的人。
$ P +提高了$ P Point-Cloud识别器的准确性。
$ P +是通过仔细研究低视力人群的中风姿势表现而开发的，这为如何为所有用户提高$ P提供了见解。
 */

const NUM_POINTS: usize = 32;
const ORIGIN: Point = Point {
    x: 0.0,
    y: 0.0,
    id: 0,
    angle: 0.0,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub id: usize,
    pub angle: f64, // normalized turning angle, $P+
}

impl Point {
    pub fn new<T: Into<f64>>(x: T, y: T, id: usize) -> Point {
        Point {
            x: x.into(),
            y: y.into(),
            id,
            angle: 0.0,
        }
    }

    pub fn with_angle<T: Into<f64>>(x: T, y: T, id: usize, angle: f64) -> Point {
        Point {
            x: x.into(),
            y: y.into(),
            id,
            angle,
        }
    }
}

//
// Result class
//
pub struct Result {
    pub name: String,
    pub score: f64,
    pub ms: f64,
}

impl Result {
    fn new(name: &str, score: f64, ms: f64) -> Result {
        // constructor
        Result {
            name: name.to_string(),
            score,
            ms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PointCloud {
    name: String,
    pub points: Vec<Point>,
}

impl PointCloud {
    fn new(name: &str, points: Vec<Point>) -> PointCloud {
        let points = resample(points, NUM_POINTS);
        let points = scale(&points);
        let points = translate_to(&points, &ORIGIN);
        PointCloud {
            name: name.to_string(),
            points: compute_normalized_turning_angle(&points),
        }
    }
}

impl Default for PointCloud {
    fn default() -> PointCloud {
        PointCloud {
            name: String::new(),
            points: vec![],
        }
    }
}

pub struct PDollarPlusRecognizer {
    point_clouds: Vec<PointCloud>,
}

impl PDollarPlusRecognizer {
    pub fn new() -> PDollarPlusRecognizer {
        PDollarPlusRecognizer {
            point_clouds: vec![],
        }
    }

    pub fn recognize(&self, points: Vec<Point>) -> Result {
        //let t0 = Instant::now();
        let points = translate_to(&scale(&resample(points, NUM_POINTS)), &ORIGIN);
        let points = compute_normalized_turning_angle(&points); // $P+

        let mut b = std::f64::MAX;
        let mut u = -1;
        for i in 0..self.point_clouds.len() {
            // for each point-cloud template
            let d = cloud_distance(&points, &self.point_clouds[i].points)
                .min(cloud_distance(&self.point_clouds[i].points, &points)); // $P+
            if d < b {
                b = d; // best (least) distance
                u = i as i32; // point-cloud index
            }
        }

        //let t1 = duration_to_milis(&t0.elapsed());
        let t1 = 0.0;

        if u == -1 {
            Result::new("No match.", -1.0, t1)
        } else {
            Result::new(
                &self.point_clouds[u as usize].name,
                b, // $P+
                t1,
            )
        }
    }

    pub fn add_gesture(&mut self, name: &str, points: Vec<Point>) -> usize {
        //println!("add_gesture name={}", name);
        self.point_clouds.push(PointCloud::new(name, points));
        let mut num = 0;
        for i in 0..self.point_clouds.len() {
            if self.point_clouds[i].name == name {
                num += 1;
            }
        }
        num
    }

    pub fn clear_gestures(&mut self) {
        self.point_clouds.clear();
    }

    pub fn _point_clouds(&self) -> &Vec<PointCloud> {
        &self.point_clouds
    }
}

fn cloud_distance(pts1: &Vec<Point>, pts2: &Vec<Point>) -> f64 {
    // revised for $P+
    let mut matched = vec![false; pts1.len()]; // pts1.length == pts2.length
    let mut sum = 0.0;
    for i in 0..pts1.len() {
        let mut index = -1;
        let mut min = std::f64::MAX;
        for j in 0..pts1.len() {
            let d = distance_with_angle(&pts1[i], &pts2[j]);
            if d < min {
                min = d;
                index = j as i32;
            }
        }
        matched[index as usize] = true;
        sum += min;
    }
    for j in 0..matched.len() {
        if !matched[j] {
            let mut min = std::f64::MAX;
            for i in 0..pts1.len() {
                let d = distance_with_angle(&pts1[i], &pts2[j]);
                if d < min {
                    min = d;
                }
            }
            sum += min;
        }
    }
    return sum;
}

fn distance_with_angle(p1: &Point, p2: &Point) -> f64 {
    // $P+
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let da = p2.angle - p1.angle;
    (dx * dx + dy * dy + da * da).sqrt()
}

fn compute_normalized_turning_angle(points: &Vec<Point>) -> Vec<Point> {
    // $P+
    let mut newpoints = vec![];
    newpoints.push(Point::new(points[0].x, points[0].y, points[0].id)); // first point
    for i in 1..points.len() - 1 {
        let dx = (points[i + 1].x - points[i].x) * (points[i].x - points[i - 1].x);
        let dy = (points[i + 1].y - points[i].y) * (points[i].y - points[i - 1].y);
        let dn = distance(&points[i + 1], &points[i]) * distance(&points[i], &points[i - 1]);
        let cosangle = (-1.0f64).max(1.0f64.min((dx + dy) / dn)); // ensure [-1,+1]
        let angle = cosangle.acos() / PI; // normalized angle
        newpoints.push(Point::with_angle(
            points[i].x,
            points[i].y,
            points[i].id,
            angle,
        ));
    }
    newpoints.push(Point::new(
        // last point
        points[points.len() - 1].x,
        points[points.len() - 1].y,
        points[points.len() - 1].id,
    ));
    newpoints
}

fn translate_to(points: &Vec<Point>, pt: &Point) -> Vec<Point> {
    // translates points' centroid
    let c = centroid(points);
    let mut newpoints = vec![];
    for point in points {
        let qx = point.x + pt.x - c.x;
        let qy = point.y + pt.y - c.y;
        newpoints.push(Point::new(qx, qy, point.id));
    }
    newpoints
}

fn centroid(points: &Vec<Point>) -> Point {
    let mut x = 0.0;
    let mut y = 0.0;
    for point in points {
        x += point.x;
        y += point.y;
    }
    x /= points.len() as f64;
    y /= points.len() as f64;
    Point::new(x, y, 0)
}

fn scale(points: &Vec<Point>) -> Vec<Point> {
    let mut min_x = std::f64::MAX;
    let mut max_x = std::f64::MIN;
    let mut min_y = std::f64::MAX;
    let mut max_y = std::f64::MIN;
    for i in 0..points.len() {
        min_x = min_x.min(points[i].x);
        min_y = min_y.min(points[i].y);
        max_x = max_x.max(points[i].x);
        max_y = max_y.max(points[i].y);
    }
    let size = (max_x - min_x).max(max_y - min_y);
    let mut new_points = vec![];
    for i in 0..points.len() {
        let qx = (points[i].x - min_x) / size;
        let qy = (points[i].y - min_y) / size;
        new_points.push(Point::new(qx, qy, points[i].id));
    }
    new_points
}

pub fn resample(mut points: Vec<Point>, n: usize) -> Vec<Point> {
    let len = path_length(&points) / (n as f64 - 1.0); // interval length
    let mut dist = 0.0;
    let mut new_points = vec![points[0].clone()];

    let mut i = 1;
    while i < points.len() {
        if points[i].id == points[i - 1].id {
            let d = distance(&points[i - 1], &points[i]);
            if (dist + d) >= len {
                let qx = points[i - 1].x + ((len - dist) / d) * (points[i].x - points[i - 1].x);
                let qy = points[i - 1].y + ((len - dist) / d) * (points[i].y - points[i - 1].y);
                let q = Point::new(qx, qy, points[i].id);
                new_points.push(q.clone()); // append Point::new 'q'
                points.insert(i, q); // insert 'q' at position i in points s.t. 'q' will be the next i
                dist = 0.0;
            } else {
                dist += d;
            }
        }
        i += 1;
    }
    if new_points.len() == n as usize - 1 {
        // somtimes we fall a rounding-error short of adding the last point, so add it if so
        new_points.push(Point::new(
            points[points.len() - 1].x,
            points[points.len() - 1].y,
            points[points.len() - 1].id,
        ));
    }

    //println!("resample之后:{:?}", new_points);

    new_points
}

// length traversed by a point path
fn path_length(points: &Vec<Point>) -> f64 {
    let mut d = 0.0;
    for i in 1..points.len() {
        if points[i].id == points[i - 1].id {
            d += distance(&points[i - 1], &points[i]);
        }
    }
    d
}

pub fn distance(p1: &Point, p2: &Point) -> f64 {
    // Euclidean distance between two points
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

// pub fn duration_to_milis(duration: &Duration) -> f64 {
//     duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
// }
