use crate::history::{HistoryBuffer, Transaction, TransactionKind};
use crate::ledger::Ledger;
use crate::management;
use crate::stats::StatsData;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use std::collections::BTreeMap;
use std::iter::FromIterator;

fn get_data() -> BTreeMap<usize, Principal> {
    let data: Vec<(usize, &str)> = vec![
        (
            4,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            6,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            8,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            9,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            15,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            19,
            "krrme-whga7-5wkan-xsrle-3fsck-u5qm6-yisle-7rc66-linbc-4cpdz-vqe",
        ),
        (
            20,
            "krrme-whga7-5wkan-xsrle-3fsck-u5qm6-yisle-7rc66-linbc-4cpdz-vqe",
        ),
        (
            23,
            "yfkjr-g7wsz-27ppq-fnomj-d76ib-h3pwj-4p52m-mcloq-e53hn-asgqv-4ae",
        ),
        (
            24,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            25,
            "mwp5g-agwxm-genl7-rh5gi-4nbla-e5mn5-6bkj4-3ellr-qlq4m-npwsx-sae",
        ),
        (
            27,
            "yfkjr-g7wsz-27ppq-fnomj-d76ib-h3pwj-4p52m-mcloq-e53hn-asgqv-4ae",
        ),
        (
            73,
            "ztvb3-4eqq3-hgw7z-5tvpz-hj56w-6cku4-irkn2-toj63-yerz2-4wflg-sae",
        ),
        (
            74,
            "ztvb3-4eqq3-hgw7z-5tvpz-hj56w-6cku4-irkn2-toj63-yerz2-4wflg-sae",
        ),
        (
            81,
            "pjlvn-lvzf6-lnc2n-ygvhm-2giiq-sp36q-ak6oc-h3obc-j2kb7-axbqo-3qe",
        ),
        (
            82,
            "pjlvn-lvzf6-lnc2n-ygvhm-2giiq-sp36q-ak6oc-h3obc-j2kb7-axbqo-3qe",
        ),
        (
            114,
            "uhubn-2jqzt-y3ifi-ykwq2-wctcc-s32sa-srkly-4mmzd-rdr6f-5wwsm-lqe",
        ),
        (
            115,
            "uhubn-2jqzt-y3ifi-ykwq2-wctcc-s32sa-srkly-4mmzd-rdr6f-5wwsm-lqe",
        ),
        (
            126,
            "qtymw-i2owz-z53am-5ywzn-e2yat-aibmp-fwurc-hemdv-qy7lb-4y2tt-gae",
        ),
        (
            128,
            "qtymw-i2owz-z53am-5ywzn-e2yat-aibmp-fwurc-hemdv-qy7lb-4y2tt-gae",
        ),
        (
            132,
            "uhubn-2jqzt-y3ifi-ykwq2-wctcc-s32sa-srkly-4mmzd-rdr6f-5wwsm-lqe",
        ),
        (
            133,
            "uhubn-2jqzt-y3ifi-ykwq2-wctcc-s32sa-srkly-4mmzd-rdr6f-5wwsm-lqe",
        ),
        (
            134,
            "uhubn-2jqzt-y3ifi-ykwq2-wctcc-s32sa-srkly-4mmzd-rdr6f-5wwsm-lqe",
        ),
        (
            152,
            "5ml63-jvxh6-kjwcq-2vkof-keyrx-h2hh3-aa5f7-eehv3-cxkrq-xo5kd-bae",
        ),
        (
            153,
            "5ml63-jvxh6-kjwcq-2vkof-keyrx-h2hh3-aa5f7-eehv3-cxkrq-xo5kd-bae",
        ),
        (
            178,
            "3kift-624gy-aj2yt-6azao-3dzwv-k5juf-r5qvo-5h4jb-ixycl-2jphq-cae",
        ),
        (
            179,
            "3kift-624gy-aj2yt-6azao-3dzwv-k5juf-r5qvo-5h4jb-ixycl-2jphq-cae",
        ),
        (
            195,
            "tmxuf-27tam-c4rtu-autyv-co5kl-fzdul-cnbth-tuwrt-6d7pa-vipge-bqe",
        ),
        (
            196,
            "tmxuf-27tam-c4rtu-autyv-co5kl-fzdul-cnbth-tuwrt-6d7pa-vipge-bqe",
        ),
        (
            197,
            "tmxuf-27tam-c4rtu-autyv-co5kl-fzdul-cnbth-tuwrt-6d7pa-vipge-bqe",
        ),
        (
            198,
            "tmxuf-27tam-c4rtu-autyv-co5kl-fzdul-cnbth-tuwrt-6d7pa-vipge-bqe",
        ),
        (
            249,
            "h7iva-6kqo6-2zfh3-x7k4o-zfc4q-qhp7n-7l6iu-np3j3-exo7b-zzts3-fqe",
        ),
        (
            250,
            "h7iva-6kqo6-2zfh3-x7k4o-zfc4q-qhp7n-7l6iu-np3j3-exo7b-zzts3-fqe",
        ),
        (
            258,
            "lhwkg-2iarf-jb5l4-txjoq-7v3gk-ra5n4-nhkmn-qrksb-i32hw-ej5h2-hqe",
        ),
        (
            267,
            "zf3yj-a5rb3-cbnet-sfqaa-3kvsl-2b723-lw6bh-czuy4-oshge-kylfx-2qe",
        ),
        (
            269,
            "bdkre-n4q7k-bpj76-mu3g3-nvgdk-327y7-ger4q-itwcr-kfhv3-ncmx2-oae",
        ),
        (
            275,
            "3j3gn-3yzy7-6fcna-i6hsy-hddba-y7x5x-lkham-wkqa2-77luf-jyyks-uqe",
        ),
        (
            305,
            "yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae",
        ),
        (
            306,
            "yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae",
        ),
        (
            307,
            "yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae",
        ),
        (
            339,
            "l7jxi-2lmoh-t4lmj-5vcwz-2f4ir-7oslh-hkvd7-srcz5-63rys-hwc65-wae",
        ),
        (
            340,
            "l7jxi-2lmoh-t4lmj-5vcwz-2f4ir-7oslh-hkvd7-srcz5-63rys-hwc65-wae",
        ),
        (
            394,
            "h7l3t-b2s7e-h67e4-nnvfj-y33j3-wueex-mahcy-vb44b-7q7e3-g2zbi-cqe",
        ),
        (
            395,
            "h7l3t-b2s7e-h67e4-nnvfj-y33j3-wueex-mahcy-vb44b-7q7e3-g2zbi-cqe",
        ),
        (
            674,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            675,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            676,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            885,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            886,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            889,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            890,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            891,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            892,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            893,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            894,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            895,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            896,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            897,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            899,
            "3z4ue-dry77-pvwdh-4ugn3-lu2wi-sbfp6-7xzaf-jupqw-vqiit-zi7m7-gae",
        ),
        (
            900,
            "3z4ue-dry77-pvwdh-4ugn3-lu2wi-sbfp6-7xzaf-jupqw-vqiit-zi7m7-gae",
        ),
        (
            902,
            "siay4-pltgm-xo7wa-sl2xv-dwrtw-jobvb-2epkw-dpvin-girai-pe7ts-mae",
        ),
        (
            904,
            "siay4-pltgm-xo7wa-sl2xv-dwrtw-jobvb-2epkw-dpvin-girai-pe7ts-mae",
        ),
        (
            991,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            992,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            993,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            994,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            995,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            996,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            999,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            1000,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            1001,
            "qtli6-tgvjq-v6po2-53nmj-pao4e-ym435-cujqe-6456w-wr7uw-bsic3-gqe",
        ),
        (
            1003,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            1004,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            1005,
            "gehnv-d2urf-4mg3e-cioxw-xs2bk-nwxx5-c2pif-pfyf2-7t4ba-dipec-xae",
        ),
        (
            1006,
            "gehnv-d2urf-4mg3e-cioxw-xs2bk-nwxx5-c2pif-pfyf2-7t4ba-dipec-xae",
        ),
        (
            1007,
            "gehnv-d2urf-4mg3e-cioxw-xs2bk-nwxx5-c2pif-pfyf2-7t4ba-dipec-xae",
        ),
        (
            1014,
            "3hr2m-r64pk-3wlc5-2kxpo-ru6pd-cfktv-7hzff-omvjf-xg7np-u77ak-pae",
        ),
        (
            1015,
            "3hr2m-r64pk-3wlc5-2kxpo-ru6pd-cfktv-7hzff-omvjf-xg7np-u77ak-pae",
        ),
        (
            1025,
            "y4tdy-oqh5k-fgmxy-nma7k-wvldx-kgavw-m5vo2-izprf-nz2vp-cn6qu-bqe",
        ),
        (
            1028,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            1029,
            "a27dj-2h26t-rsqtr-d5qxj-px4p4-y532n-gbrcx-sp3ew-ujd3u-uukxa-dqe",
        ),
        (
            1034,
            "gehnv-d2urf-4mg3e-cioxw-xs2bk-nwxx5-c2pif-pfyf2-7t4ba-dipec-xae",
        ),
        (
            1035,
            "gehnv-d2urf-4mg3e-cioxw-xs2bk-nwxx5-c2pif-pfyf2-7t4ba-dipec-xae",
        ),
        (
            1085,
            "l3hrb-vboyy-2kyuv-avynm-qmq62-facvc-e44bd-hputk-ivn2q-4onyz-4qe",
        ),
        (
            1086,
            "l3hrb-vboyy-2kyuv-avynm-qmq62-facvc-e44bd-hputk-ivn2q-4onyz-4qe",
        ),
        (
            1101,
            "yueug-uayck-ofnsa-26x36-lr5ic-gtgkf-ktnue-sbney-6y5vy-y7kqp-6ae",
        ),
        (
            1102,
            "yueug-uayck-ofnsa-26x36-lr5ic-gtgkf-ktnue-sbney-6y5vy-y7kqp-6ae",
        ),
        (
            1109,
            "mlx7d-nlzwm-jsiyr-txxc2-mlgsf-hafo6-73wnd-du4xx-f2tsd-mjtum-pae",
        ),
    ];

    BTreeMap::from_iter(
        data.into_iter()
            .map(|(k, v)| (k, Principal::from_text(v).unwrap())),
    )
}

#[derive(CandidType, Clone, Deserialize)]
enum TransactionKindV0 {
    Transfer {
        from: Principal,
        to: Principal,
    },
    Mint {
        to: Principal,
    },
    Burn {
        from: Principal,
        to: Principal,
    },
    CanisterCalled {
        canister: Principal,
        method_name: String,
    },
    CanisterCreated {
        canister: Principal,
    },
    ChargingStationDeployed {
        canister: Principal,
    },
}

#[derive(CandidType, Clone, Deserialize)]
struct TransactionV0 {
    pub timestamp: u64,
    pub cycles: u64,
    pub fee: u64,
    pub kind: TransactionKindV0,
}

impl From<TransactionV0> for Transaction {
    fn from(tx: TransactionV0) -> Self {
        Transaction {
            timestamp: tx.timestamp,
            cycles: tx.cycles,
            fee: tx.fee,
            kind: tx.kind.into(),
        }
    }
}

impl From<TransactionKindV0> for TransactionKind {
    fn from(tx: TransactionKindV0) -> Self {
        match tx {
            TransactionKindV0::Transfer { from, to } => TransactionKind::Transfer { from, to },
            TransactionKindV0::Mint { to } => TransactionKind::Mint { to },
            TransactionKindV0::Burn { from, to } => TransactionKind::Burn { from, to },
            TransactionKindV0::CanisterCalled {
                canister,
                method_name,
            } => TransactionKind::CanisterCalled {
                from: Principal::anonymous(),
                canister,
                method_name,
            },
            TransactionKindV0::CanisterCreated { canister } => TransactionKind::CanisterCreated {
                from: Principal::anonymous(),
                canister,
            },
            TransactionKindV0::ChargingStationDeployed { canister } => {
                TransactionKind::ChargingStationDeployed {
                    from: Principal::anonymous(),
                    canister,
                }
            }
        }
    }
}

#[derive(CandidType, Deserialize)]
struct StableStorageV0 {
    ledger: Vec<(Principal, u64)>,
    history: Vec<TransactionV0>,
    controller: Principal,
    stats: StatsData,
}

#[derive(CandidType, Deserialize)]
struct StableStorage {
    ledger: Vec<(Principal, u64)>,
    history: Vec<Transaction>,
    controller: Principal,
    stats: StatsData,
}

#[pre_upgrade]
pub fn pre_upgrade() {
    let ledger = storage::get_mut::<Ledger>().archive();
    let history = storage::get_mut::<HistoryBuffer>().archive();
    let controller = management::Controller::get_principal();

    let stable = StableStorage {
        ledger,
        history,
        controller,
        stats: StatsData::get(),
    };

    match storage::stable_save((stable,)) {
        Ok(_) => (),
        Err(candid_err) => {
            trap(&format!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            ));
        }
    };
}

#[post_upgrade]
pub fn post_upgrade() {
    if let Ok((stable,)) = storage::stable_restore::<(StableStorageV0,)>() {
        let data = get_data();
        let mut history = Vec::with_capacity(stable.history.len());
        for (index, event) in stable.history.into_iter().enumerate() {
            let mut e: Transaction = event.into();

            if let Some(archive_from) = data.get(&index) {
                match &mut e.kind {
                    TransactionKind::Transfer { .. } => unreachable!(),
                    TransactionKind::Mint { .. } => unreachable!(),
                    TransactionKind::Burn { .. } => unreachable!(),
                    TransactionKind::CanisterCalled { from, .. } => {
                        *from = archive_from.clone();
                    }
                    TransactionKind::CanisterCreated { from, .. } => {
                        *from = archive_from.clone();
                    }
                    TransactionKind::ChargingStationDeployed { from, .. } => {
                        *from = archive_from.clone();
                    }
                }
            }

            history.push(e)
        }

        storage::get_mut::<Ledger>().load(stable.ledger);
        storage::get_mut::<HistoryBuffer>().load(history);
        management::Controller::load(stable.controller);
        StatsData::load(stable.stats);
    }
}
