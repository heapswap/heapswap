use crate::*;


pub trait ChordTrait<ChordNode> {
	// Pseudocode

	// Definitions for pseudocode

	// 	original definition:
	//     finger[k]
	//         first node that succeeds ( n + 2^(k − 1))  mod 2^m , 1 ≤ k ≤ m	
	//  modified definition:
	//     finger[k]
	//         first node that succeeds (n + 2^k)  mod 2^m, 0 ≤ k ≤ m-1	

	//     successor
	//         the next node from the node in question on the identifier ring
	//     predecessor
	//         the previous node from the node in question on the identifier ring

	// The pseudocode to find the successor node of an id is given below:

	// // ask node n to find the successor of id
	// n.find_successor(id)
	//     // Yes, that should be a closing square bracket to match the opening parenthesis.
	//     // It is a half closed interval.
	//     if id ∈ (n, successor] then
	//         return successor
	//     else
	//         // forward the query around the circle
	//         n0 := closest_preceding_node(id)
	//         return n0.find_successor(id)
	fn find_successor(&self, id: &ChordId) -> ChordId;

	// // search the local table for the highest predecessor of id
	// n.closest_preceding_node(id)
	//     for i = m downto 1 do
	//         if (finger[i] ∈ (n, id)) then
	//             return finger[i]
	//     return n
	fn closest_preceding_node(&self, id: &ChordId) -> ChordId;

	// The pseudocode to stabilize the chord ring/circle after node joins and departures is as follows:

	// // create a new Chord ring.
	// n.create()
	//     predecessor := nil
	//     successor := n
	fn create(&self) -> ChordNode;

	// // join a Chord ring containing node n'.
	// n.join(n')
	//     predecessor := nil
	//     successor := n'.find_successor(n)
	fn join(&self, n: ChordNode) -> ChordNode;

	// // called periodically. n asks the successor
	// // about its predecessor, verifies if n's immediate
	// // successor is consistent, and tells the successor about n
	// n.stabilize()
	//     x = successor.predecessor
	//     if x ∈ (n, successor) then
	//         successor := x
	//     successor.notify(n)
	fn stabilize(&self) -> ChordNode;

	// // n' thinks it might be our predecessor.
	// n.notify(n')
	//     if predecessor is nil or n'∈(predecessor, n) then
	//         predecessor := n'
	fn notify(&self, n: ChordNode) -> ChordNode;

	// // called periodically. refreshes finger table entries.
	// // next stores the index of the finger to fix
	// n.fix_fingers()
	//     next := next + 1
	//     if next > m then
	//         next := 1
	//     finger[next] := find_successor(n+2next-1);
	fn fix_fingers(&self) -> ChordNode;

	// // called periodically. checks whether predecessor has failed.
	// n.check_predecessor()
	//     if predecessor has failed then
	//         predecessor := nil
	fn check_predecessor(&self) -> ChordNode;
}
