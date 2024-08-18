// use super::*;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

pub type Transmitter<T> = UnboundedSender<T>;
pub type Receiver<R> = UnboundedReceiver<R>;

/**
 * Unidirectional channel
*/
pub fn portal<U>() -> (Transmitter<U>, Receiver<U>) {
	let (tx, rx) = unbounded::<U>();
	(tx, rx)
}

pub fn portals<L, R>() -> (Portal<R, L>, Portal<L, R>)
where
	L: Clone,
	R: Clone,
{
	Portals::new().split()
}

pub struct Portal<T, R>
where
	T: Clone,
	R: Clone,
{
	tx: Transmitter<T>,
	tx_self: Transmitter<R>,
	rx: Receiver<R>,
}

impl<T, R> Portal<T, R>
where
	T: Clone,
	R: Clone,
{
	pub fn tx(&self) -> Transmitter<T> {
		self.tx.clone()
	}

	pub fn tx_self(&self) -> Transmitter<R> {
		self.tx_self.clone()
	}

	pub fn rx(&mut self) -> &mut Receiver<R> {
		&mut self.rx
	}
}

/**
 * Bidirectional channel
*/
struct Portals<L, R>
where
	L: Clone,
	R: Clone,
{
	left: Portal<R, L>,
	right: Portal<L, R>,
}

impl<L, R> Portals<L, R>
where
	L: Clone,
	R: Clone,
{
	pub fn new() -> Self {
		let (tx_left, rx_left) = portal::<L>();
		let (tx_right, rx_right) = portal::<R>();

		Self {
			left: Portal {
				tx: tx_right.clone(),
				tx_self: tx_left.clone(),
				rx: rx_left,
			},
			right: Portal {
				tx: tx_left.clone(),
				tx_self: tx_right.clone(),
				rx: rx_right,
			},
		}
	}

	pub fn split(self) -> (Portal<R, L>, Portal<L, R>) {
		(self.left, self.right)
	}
}
