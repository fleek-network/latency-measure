use std::ops::{Deref, DerefMut};
use measure::MeasureResponse;
use serde::{Deserialize, Serialize};


fn average(items: &Vec<impl Deref<Target = MeasureResponse>>, times: usize) -> MeasureResponse {
    let starting = MeasureResponse {
        dns_lookup_duration: Some(Default::default()),
        tcp_connect_duration: Default::default(),
        http_get_send_duration: Default::default(),
        ttfb_duration: Default::default(),
        tls_handshake_duration: Some(Default::default()),
    };

    let mut summed = items.iter().fold(starting, |mut init, val| {
        if let Some(dur) = val.dns_lookup_duration {
            if init.dns_lookup_duration.is_some() {
                init.dns_lookup_duration = Some(init.dns_lookup_duration.unwrap() + dur);
            }
        } else {
            init.dns_lookup_duration = None;
        }

        if let Some(dur) = val.tls_handshake_duration {
            if init.tls_handshake_duration.is_some() {
                init.tls_handshake_duration = Some(init.tls_handshake_duration.unwrap() + dur);
            }
        } else {
            init.tls_handshake_duration = None;
        }

        init.tcp_connect_duration += val.tcp_connect_duration;
        init.http_get_send_duration += val.http_get_send_duration;
        init.ttfb_duration += val.ttfb_duration;

        init
    });

    if let Some(dur) = summed.dns_lookup_duration {
        summed.dns_lookup_duration = Some(dur / times as u32);
    }

    if let Some(dur) = summed.tls_handshake_duration {
        summed.tls_handshake_duration = Some(dur / times as u32);
    }

    summed.tcp_connect_duration /= times as u32;
    summed.http_get_send_duration /= times as u32;
    summed.ttfb_duration /= times as u32;

    summed
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Labeled {
    label: String,
    inner: MeasureResponse,
}

impl Labeled {
    pub fn print(&self) {
        println!("{:#?}", self);
    }

    pub fn new(inner: MeasureResponse, label: String) -> Self {
        Self { label, inner }
    }

    pub fn average(items: &Vec<Self>, times: usize) -> MeasureResponse {
        average(&items, times)
    }
}

impl Deref for Labeled {
    type Target = MeasureResponse;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Labeled {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}