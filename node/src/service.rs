//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use sc_client::LongestChain;
use sc_client_api::ExecutorProvider;
use runtime::{self, opaque::Block, RuntimeApi};
use sc_service::{error::{Error as ServiceError}, AbstractService, Configuration, ServiceBuilder};
use sp_inherents::InherentDataProviders;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sha3pow::Sha3Algorithm;
use sc_network::{config::DummyFinalityProofRequestBuilder};
use core::clone::Clone;
use sp_core::sr25519;
use parity_scale_codec::Encode;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	runtime::api::dispatch,
	runtime::native_version,
);

pub fn build_inherent_data_providers(sr25519_public_key: sr25519::Public) -> Result<InherentDataProviders, ServiceError> {
	let providers = InherentDataProviders::new();

	providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.map_err(Into::into)
		.map_err(sp_consensus::error::Error::InherentData)?;


	//if let Some(author) = author
	providers
		.register_provider(runtime::block_author::InherentDataProvider(
			sr25519_public_key.encode(),
		))
		.map_err(Into::into)
		.map_err(sp_consensus::error::Error::InherentData)?;

	Ok(providers)
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr, $sr25519_public_key:expr) => {{
		let mut import_setup = None;
		let inherent_data_providers = crate::service::build_inherent_data_providers($sr25519_public_key)?;

		let builder = sc_service::ServiceBuilder::new_full::<
			runtime::opaque::Block, runtime::RuntimeApi, crate::service::Executor
		>($config)?
			.with_select_chain(|_config, backend| {
				Ok(sc_client::LongestChain::new(backend.clone()))
			})?
			.with_transaction_pool(|config, client, _fetcher| {
				let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
				Ok(sc_transaction_pool::BasicPool::new(config, std::sync::Arc::new(pool_api)))
			})?
			.with_import_queue(|_config, client, select_chain, _transaction_pool| {

				let pow_block_import = sc_consensus_pow::PowBlockImport::new(
					client.clone(),
					client.clone(),
					sha3pow::Sha3Algorithm::new(client.clone()),
					0, // check inherents starting at block 0
					select_chain,
					inherent_data_providers.clone(),
				);

				let import_queue = sc_consensus_pow::import_queue(
					Box::new(pow_block_import.clone()),
					sha3pow::Sha3Algorithm::new(client.clone()),
					inherent_data_providers.clone(),
				)?;

				import_setup = Some(pow_block_import);

				Ok(import_queue)
			})?;

		(builder, import_setup, inherent_data_providers)
	}}
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration, sr25519_public_key: sr25519::Public)
	-> Result<impl AbstractService, ServiceError>
{
	let participates_in_consensus = config.role.is_authority();

	let (builder, mut import_setup, inherent_data_providers) = new_full_start!(config, sr25519_public_key);
	let block_import = import_setup.take()
		.expect("Block Import is present for Full Services or setup failed before. qed");

	let service = builder.build()?;

	if participates_in_consensus {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			service.client(),
			service.transaction_pool()
		);

		// The number of rounds of mining to try in a single call
		let rounds = 500;

		let client = service.client();
		let select_chain = service.select_chain()
			.ok_or(ServiceError::SelectChainRequired)?;

		let can_author_with =
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

		sc_consensus_pow::start_mine(
			Box::new(block_import),
			client.clone(),
			Sha3Algorithm::new(client.clone()),
			proposer,
			None, // No preruntime digests
			rounds,
			service.network(),
			std::time::Duration::new(2, 0),
			// Choosing not to supply a select_chain means we will use the client's
			// possibly-outdated metadata when fetching the block to mine on
			Some(select_chain),
			inherent_data_providers.clone(),
			can_author_with,
		);
	}

	Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration, sr25519_public_key: sr25519::Public)
	-> Result<impl AbstractService, ServiceError>
{
	ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
		.with_select_chain(|_config, backend| {
			Ok(LongestChain::new(backend.clone()))
		})?
		.with_transaction_pool(|config, client, fetcher| {
			let fetcher = fetcher
				.ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;

			let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
			let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
				config, Arc::new(pool_api), sc_transaction_pool::RevalidationType::Light,
			);
			Ok(pool)
		})?
		.with_import_queue_and_fprb(|_config, client, _backend, _fetcher, select_chain, _tx_pool| {
			let finality_proof_request_builder =
				Box::new(DummyFinalityProofRequestBuilder::default()) as Box<_>;

			let pow_block_import = sc_consensus_pow::PowBlockImport::new(
				client.clone(),
				client.clone(),
				Sha3Algorithm::new(client.clone()),
				0, // check_inherents_after,
				select_chain,
				build_inherent_data_providers(sr25519_public_key)?,
			);

			let import_queue = sc_consensus_pow::import_queue(
				Box::new(pow_block_import),
				Sha3Algorithm::new(client),
				build_inherent_data_providers(sr25519_public_key)?,
			)?;

			Ok((import_queue, finality_proof_request_builder))
		})?
		.with_finality_proof_provider(|_client, _backend|
			Ok(Arc::new(()) as _)
		)?
		.build()
}
