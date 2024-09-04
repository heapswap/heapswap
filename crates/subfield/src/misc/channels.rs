use crate::*;

pub type Sender<T> = UnboundedSender<T>;
pub type Receiver<R> = UnboundedReceiver<R>;

/*
   Unidirectional channel
*/
pub fn channel<U>() -> (Sender<U>, Receiver<U>) {
	let (tx, rx) = unbounded::<U>();
	(tx, rx)
}

pub fn channels<L, R>() -> (Unichannel<R, L>, Unichannel<L, R>)
where
	L: Clone,
	R: Clone,
{
	Bichannel::new().split()
}

pub struct Unichannel<T, R>
where
	T: Clone,
	R: Clone,
{
	tx: Sender<T>,
	tx_self: Sender<R>,
	rx: Receiver<R>,
}

impl<T, R> Unichannel<T, R>
where
	T: Clone,
	R: Clone,
{
	pub fn tx(&self) -> Sender<T> {
		self.tx.clone()
	}

	pub fn tx_self(&self) -> Sender<R> {
		self.tx_self.clone()
	}

	pub fn rx(&mut self) -> &mut Receiver<R> {
		&mut self.rx
	}
}

/*
   Bidirectional channel
*/
struct Bichannel<L, R>
where
	L: Clone,
	R: Clone,
{
	left: Unichannel<R, L>,
	right: Unichannel<L, R>,
}

impl<L, R> Bichannel<L, R>
where
	L: Clone,
	R: Clone,
{
	pub fn new() -> Self {
		let (tx_left, rx_left) = channel::<L>();
		let (tx_right, rx_right) = channel::<R>();

		Self {
			left: Unichannel {
				tx: tx_right.clone(),
				tx_self: tx_left.clone(),
				rx: rx_left,
			},
			right: Unichannel {
				tx: tx_left.clone(),
				tx_self: tx_right.clone(),
				rx: rx_right,
			},
		}
	}

	pub fn split(self) -> (Unichannel<R, L>, Unichannel<L, R>) {
		(self.left, self.right)
	}
}
