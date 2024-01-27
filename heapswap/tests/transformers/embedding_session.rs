use heapswap::embeddings::EmbeddingSession;
use dashmap::DashMap;
use anyhow::Result;
use timeit::*;

#[test]
fn test_vector_timing() -> Result<()> {
    
   	let model_bytes = include_bytes!("../../models/gte-small/model.onnx");	
	let tokenizer_bytes = include_bytes!("../../models/gte-small/tokenizer.json");

	let session = EmbeddingSession::new(
		"gte-small",
		model_bytes,
		//model_bytes_static,
		tokenizer_bytes,
		512,
		2, //gte-small seems to have diminishing returns after 3 threads
	);

	let loop_count = 10;

	let sequence_map: DashMap<&str, &str> = DashMap::new();

	sequence_map.insert("short", "orangutans are cool");
	sequence_map.insert("medium", r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago. "#);
	sequence_map.insert("long", r#"Orangutans are great apes native to the rainforests of Indonesia and Malaysia. They are now found only in parts of Borneo and Sumatra, but during the Pleistocene they ranged throughout Southeast Asia and South China. Classified in the genus Pongo, orangutans were originally considered to be one species. From 1996, they were divided into two species: the Bornean orangutan (P. pygmaeus, with three subspecies) and the Sumatran orangutan (P. abelii). A third species, the Tapanuli orangutan (P. tapanuliensis), was identified definitively in 2017. The orangutans are the only surviving species of the subfamily Ponginae, which diverged genetically from the other hominids (gorillas, chimpanzees, and humans) between 19.3 and 15.7 million years ago.

	The most arboreal of the great apes, orangutans spend most of their time in trees. They have proportionally long arms and short legs, and have reddish-brown hair covering their bodies. Adult males weigh about 75 kg (165 lb), while females reach about 37 kg (82 lb). Dominant adult males develop distinctive cheek pads or flanges and make long calls that attract females and intimidate rivals; younger subordinate males do not and more resemble adult females. Orangutans are the most solitary of the great apes: social bonds occur primarily between mothers and their dependent offspring. Fruit is the most important component of an orangutan's diet; but they will also eat vegetation, bark, honey, insects and bird eggs. They can live over 30 years, both in the wild and in captivity.

	Orangutans are among the most intelligent primates. They use a variety of sophisticated tools and construct elaborate sleeping nests each night from branches and foliage. The apes' learning abilities have been studied extensively. There may be distinctive cultures within populations. Orangutans have been featured in literature and art since at least the 18th century, particularly in works that comment on human society. Field studies of the apes were pioneered by primatologist Birutė Galdikas and they have been kept in captive facilities around the world since at least the early 19th century.

	All three orangutan species are considered critically endangered. Human activities have caused severe declines in populations and ranges. Threats to wild orangutan populations include poaching (for bushmeat and retaliation for consuming crops), habitat destruction and deforestation (for palm oil cultivation and logging), and the illegal pet trade. Several conservation and rehabilitation organisations are dedicated to the survival of orangutans in the wild. "#);

	// timing
	for length in ["short", "medium", "long"].iter() {
		
		let mut embedding = vec![];

		let sequence = *sequence_map.get(length).unwrap().value();
		
		let sec = timeit_loops!(loop_count, {
			embedding = session.binary_quantize(
				session.embed(sequence)?,
			)?;
		});

		println!(
			"{} sequence ({} tokens) : {} loops @ {} ms per loop",
			length,
			session.count_tokens(sequence).unwrap(),
			loop_count,
			(sec as f64 * 1000.0).round()
		);

		//println!("vector binary: {}", session.display_binary(embedding.clone())?);
		println!("vector hash: {}", session.display_base64(embedding)?);
	}

	Ok(())
}
