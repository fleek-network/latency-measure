use measure::MeasureResponse;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

fn average(items: &impl Deref<Target = Vec<MeasureResponse>>, times: usize) -> MeasureResponse {
    let starting = MeasureResponse {
        dns_lookup_duration: Some(Default::default()),
        tcp_connect_duration: Default::default(),
        http_get_send_duration: Default::default(),
        ttfb_duration: Default::default(),
        tls_handshake_duration: Some(Default::default()),
        ip: String::new(),
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
    pub label: String,
    inner: Vec<MeasureResponse>,
}

impl Labeled {
    pub fn print(&self) {
        println!("-----------------------------------------------------");
        println!("URL: {:#?}", self.label);
        println!("-----------------------------------------------------");
        for (i, item) in self.inner.iter().enumerate() {
            println!("{i}: {}ms ", item.ttfb_duration.as_millis());
        }
    }

    pub fn print_comped(first: &Self, second: &Self) {
        println!("-----------------------------------------------------");
        println!("URL: {:#?}", first.label);
        println!("vs");
        println!("URL: {:#?}", second.label);
        println!("-----------------------------------------------------");
        for (i, (f, s)) in first.iter().zip(second.iter()).enumerate() {
            println!("{}:  {}ms            vs              {}ms", i, f.ttfb_duration.as_millis(), s.ttfb_duration.as_millis());
        }
    }

    pub fn new(inner: Vec<MeasureResponse>, label: String) -> Self {
        Self { label, inner }
    }

    pub fn with_capacity(label: String, capacity: usize) -> Self {
        Self {
            label,
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn average(&self) -> MeasureResponse {
        average(self, self.inner.len())
    }
}

impl Deref for Labeled {
    type Target = Vec<MeasureResponse>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Labeled {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
