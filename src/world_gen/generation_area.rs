#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub min: (f32, f32),
    pub max: (f32, f32),
}

impl Rect {
    pub fn new(min: (f32, f32), max: (f32, f32)) -> Self {
        Self { min, max }
    }

    pub fn w(&self) -> f32 {
        self.max.0 - self.min.0
    }

    pub fn h(&self) -> f32 {
        self.max.1 - self.min.1
    }

    pub fn min_x(&self) -> f32 {
        self.min.0
    }

    pub fn min_y(&self) -> f32 {
        self.min.1
    }

    pub fn max_x(&self) -> f32 {
        self.max.0
    }

    pub fn max_y(&self) -> f32 {
        self.max.1
    }

    pub fn area(&self) -> f32 {
        self.w() * self.h()
    }

    pub fn clamp(&self, other: &Rect) -> Rect {
        Rect::new(
            (
                self.min_x().max(other.min_x()),
                self.min_y().max(other.min_y()),
            ),
            (
                self.max_x().min(other.max_x()),
                self.max_y().min(other.max_y()),
            ),
        )
    }

    pub fn contains(&self, other: &Rect) -> bool {
        &self.clamp(other) == other
    }

    pub fn removed(self, removal: &Rect) -> Vec<Rect> {
        let cont = self.clamp(removal);
        if cont.area() == 0. {
            return vec![self];
        }
        // Base case
        if cont == self {
            return Vec::new();
        }
        let mut rects = Vec::new();
        let cut = {
            // Cut from the top & recurse.
            if self.min_y() < cont.min_y() {
                // println!("cut from top");
                rects.push(Rect::new(self.min, (self.max_x(), cont.min_y())));
                Rect::new((self.min_x(), cont.min_y()), self.max)
            }
            // Cut from the bottom & recurse.
            else if self.max_y() > cont.max_y() {
                // println!("cut from bottom");
                rects.push(Rect::new((self.min_x(), cont.max_y()), self.max));
                Rect::new(self.min, (self.max_x(), cont.max_y()))
            }
            // Cut from the left & recurse.
            else if self.min_x() < cont.min_x() {
                // println!("cut from left");
                rects.push(Rect::new(self.min, (cont.min_x(), self.max_y())));
                Rect::new((cont.min_x(), self.max_y()), self.max)
            }
            // Cut from the right & recurse.
            else {
                // println!("cut from right");
                rects.push(Rect::new((cont.max_x(), self.min_y()), self.max));
                Rect::new(self.min, (cont.max_x(), self.max_y()))
            }
        };
        rects.extend(cut.removed(removal));
        return rects;
    }
}

#[derive(Clone, Debug)]
pub struct GenArea {
    available_space: Vec<Rect>,
}

impl GenArea {
    pub fn new(available_space: impl IntoIterator<Item = Rect>) -> Self {
        Self {
            available_space: available_space.into_iter().collect(),
        }
    }

    pub fn occupy(&mut self, occupation: &Rect) {
        // println!("{:?} + {:?}", self, occupation);
        // Split.
        let splitted_space: Vec<_> = self
            .available_space
            .iter()
            .cloned()
            .flat_map(|rect| rect.removed(occupation))
            .collect();
        // TODO: merge
        self.available_space = splitted_space;
    }

    pub fn can_occupy(&self, occupation: &Rect) -> bool {
        self.available_space
            .iter()
            .any(|space| space.contains(occupation))
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_area_occupy() {
        // from left full
        let mut area = GenArea::new([Rect::new((0., 0.), (100., 50.))]);
        area.occupy(&Rect::new((0., 0.), (10., 100.)));
        assert_eq!(
            area.available_space,
            vec![Rect::new((10., 0.), (100., 50.))]
        );
        // from left partial (1)
        let mut area = GenArea::new([Rect::new((0., 0.), (100., 50.))]);
        area.occupy(&Rect::new((0., 20.), (10., 50.)));
        assert_eq!(
            area.available_space,
            vec![
                Rect::new((0., 0.), (100., 20.)),
                Rect::new((10., 20.), (100., 50.))
            ]
        );
        // from left partial (2)
        let mut area = GenArea::new([Rect::new((0., 0.), (100., 50.))]);
        area.occupy(&Rect::new((0., 20.), (10., 30.)));
        assert_eq!(
            area.available_space,
            vec![
                Rect::new((0., 0.), (100., 20.)),
                Rect::new((0., 30.), (100., 50.)),
                Rect::new((10., 20.), (100., 30.))
            ]
        );
    }
}
