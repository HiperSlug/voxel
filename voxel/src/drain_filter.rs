struct DrainFilter<'a, T, F>
where 
	F: Fn(&mut T) -> bool,
{
	vec: &'a mut Vec<T>,
	pred: F,
	index: usize,
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where 
	F: Fn(&mut T) -> bool,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		while self.index < self.vec.len() {
			if (self.pred)(&mut self.vec[self.index]) {
				return Some(self.vec.swap_remove(self.index))
			} else {
				self.index += 1;
			}
		}
		None
	}
}

impl<'a, T, F> DrainFilter<'a, T, F>
where 
	F: Fn(&mut T) -> bool,
{
	pub fn new(vec: &'a mut Vec<T>, pred: F) -> Self {
		Self {
			vec,
			pred,
			index: 0
		}
	}
}
