use measure::MeasureResponse;

pub fn average<'a, I: Iterator<Item = &'a MeasureResponse>>(
    items: I,
    times: usize,
) -> MeasureResponse {
    let starting = MeasureResponse {
        dns_lookup_duration: Some(Default::default()),
        tcp_connect_duration: Default::default(),
        http_get_send_duration: Default::default(),
        ttfb_duration: Default::default(),
        tls_handshake_duration: Some(Default::default()),
        ip: String::new(),
    };

    let mut summed = items.fold(starting, |mut init, val| {
        // if a single value is None, the sum will be None
        if let Some(dur) = val.dns_lookup_duration {
            if init.dns_lookup_duration.is_some() {
                init.dns_lookup_duration = Some(init.dns_lookup_duration.unwrap() + dur);
            }
        } else {
            init.dns_lookup_duration = None;
        }

        // if a single value is None, the sum will be None
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
