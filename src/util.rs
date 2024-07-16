///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod macro_def {
	#[macro_export]
	macro_rules! count {
		($thing: expr) => { 1 };
		($($thing: expr),* $(,)?) => { 0 $(+ $crate::count!($thing))* };
	}
}
