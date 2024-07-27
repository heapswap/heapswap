use libp2p::kad::*;

pub fn handle_kad_outbound_progressed (
	id: QueryId, result: QueryResult, stats: QueryStats, step: ProgressStep,
){
	match result {
		QueryResult::GetClosestPeers(res) => {
			match res {
				Ok( GetClosestPeersOk{key, peers}) => {
					
				}
				Err(e) => {
					tracing::error!("Closest peers error: {:?}", e);
				}
			}
		}
		/*
		QueryResult::GetRecord(res) => {
			match res {
				Ok(res) => {
					match res {
						GetRecordOk::FoundRecord(record) => {
							// record found
						}
						GetRecordOk::FinishedWithNoAdditionalRecord { cache_candidates } => {
							// record not found
						}
					}
				}
				Err(e) => {
					tracing::error!("Record error: {:?}", e);
				}
			}
		}
		QueryResult::PutRecord(res) => {
			match res {
				Ok(res) => {
					// record put
				}
				Err(e) => {
					tracing::error!("Record put error: {:?}", e);
				}
			}
		}
		QueryResult::RepublishRecord(res) => {
			match res {
				Ok(res) => {
					// record republished
				}
				Err(e) => {
					tracing::error!("Record republish error: {:?}", e);
				}
			}
		}
		*/
		_ => {}
	}
	
}



pub fn handle_kad_inbound_request(request: InboundRequest) {
	match request {
		InboundRequest::GetRecord { num_closer_peers, present_locally } => {
			// handle get record request
		}
		InboundRequest::PutRecord { source, connection, record } => {
			// handle put record request
		}
		_ => {}
	}
}