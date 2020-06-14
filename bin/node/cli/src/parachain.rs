///////////////////////////////////////////////////////////////////////////////
//
//  Copyright 2018-2020 Airalab <research@aira.life>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//
///////////////////////////////////////////////////////////////////////////////
//! Robonomics Node parachain support functionality.

#[cfg(feature = "frame-benchmarking")]
pub mod executor {
    sc_executor::native_executor_instance!(
        pub Robonomics,
        robonomics_parachain_runtime::api::dispatch,
        robonomics_parachain_runtime::native_version,
        frame_benchmarking::benchmarking::HostFunctions,
    );
}

#[cfg(not(feature = "frame-benchmarking"))]
pub mod executor {
    sc_executor::native_executor_instance!(
        pub Robonomics,
        robonomics_parachain_runtime::api::dispatch,
        robonomics_parachain_runtime::native_version,
    );
}

/// Starts a `ServiceBuilder` for a full parachain service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[macro_export]
macro_rules! new_parachain {
    ($config:expr, $runtime:ty, $dispatch:ty) => {{
        use std::sync::Arc;

        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<
            node_primitives::Block,
            $runtime,
            $dispatch,
        >($config)?
        .with_select_chain(|_config, backend| Ok(sc_consensus::LongestChain::new(backend.clone())))?
        .with_transaction_pool(|config, client, _fetcher, prometheus_registry| {
            let pool_api = Arc::new(sc_transaction_pool::FullChainApi::new(client.clone()));
            Ok(sc_transaction_pool::BasicPool::new(
                config,
                pool_api,
                prometheus_registry,
            ))
        })?
        .with_import_queue(|_config, client, _, _, spawn_task_handle, registry| {
            let import_queue = cumulus_consensus::import_queue::import_queue(
                client.clone(),
                client,
                inherent_data_providers.clone(),
                spawn_task_handle,
                registry,
            )?;

            Ok(import_queue)
        })?
        .with_finality_proof_provider(|client, backend| {
            // GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
            let provider = client as Arc<dyn sc_finality_grandpa::StorageAndProofProvider<_, _>>;
            Ok(Arc::new(sc_finality_grandpa::FinalityProofProvider::new(
                backend, provider,
            )) as _)
        })?;

        inherent_data_providers
            .register_provider(sp_timestamp::InherentDataProvider)
            .expect("unable to register timestamp inherent data provider");

        (builder, inherent_data_providers)
    }};
}

pub mod chain_spec;
pub mod collator;
pub mod command;