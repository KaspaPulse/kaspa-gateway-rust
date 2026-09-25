//! Read-only Windows TCP-listener observation; never binds a production port.
use super::Result;
#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID, MIB_TCPROW_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
};

fn row_extent(count: usize, offset: usize, row_size: usize, available: usize) -> Result<usize> {
    let size = count
        .checked_mul(row_size)
        .and_then(|size| offset.checked_add(size))
        .ok_or("TCP listener table size overflow")?;
    if size > available {
        return Err("TCP listener table is truncated".to_owned());
    }
    Ok(size)
}
#[cfg(windows)]
fn rows<Row: Copy>(family: u32, offset: usize) -> Result<Vec<Row>> {
    const LIMIT: usize = 4 * 1024 * 1024;
    let mut size = 0_u32;
    // SAFETY: Windows receives a null query buffer and a valid size pointer.
    let initial = unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            0,
            family,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };
    if initial != 122 && initial != 0 {
        return Err(format!("TCP listener size query failed: {initial}"));
    }
    for _ in 0..4 {
        if size as usize > LIMIT {
            return Err("TCP listener table exceeds its bounded size".to_owned());
        }
        let mut buffer = vec![0_u8; (size as usize).max(4)];
        let mut returned = buffer.len() as u32;
        // SAFETY: The writable byte buffer is exactly returned bytes long; rows
        // are decoded with unaligned reads only after explicit bounds checks.
        let code = unsafe {
            GetExtendedTcpTable(
                buffer.as_mut_ptr().cast(),
                &mut returned,
                0,
                family,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };
        if code == 122 {
            size = returned;
            continue;
        }
        if code != 0 {
            return Err(format!("TCP listener query failed: {code}"));
        }
        if returned < 4 || returned as usize > buffer.len() {
            return Err("Invalid TCP listener buffer size".to_owned());
        }
        let count = u32::from_ne_bytes(buffer[..4].try_into().expect("four bytes")) as usize;
        row_extent(count, offset, std::mem::size_of::<Row>(), returned as usize)?;
        return Ok((0..count)
            .map(|index| {
                // SAFETY: row_extent checked the entire range; Row is a POD Win32 structure.
                unsafe {
                    std::ptr::read_unaligned(
                        buffer
                            .as_ptr()
                            .add(offset + index * std::mem::size_of::<Row>())
                            .cast::<Row>(),
                    )
                }
            })
            .collect());
    }
    Err("TCP listener table changed repeatedly during observation".to_owned())
}
#[cfg(windows)]
pub(super) fn listening(port: u16) -> Result<bool> {
    let ipv4 =
        rows::<MIB_TCPROW_OWNER_PID>(2, std::mem::offset_of!(MIB_TCPTABLE_OWNER_PID, table))?;
    if ipv4
        .iter()
        .any(|row| u16::from_be(row.dwLocalPort as u16) == port)
    {
        return Ok(true);
    }
    let ipv6 =
        rows::<MIB_TCP6ROW_OWNER_PID>(23, std::mem::offset_of!(MIB_TCP6TABLE_OWNER_PID, table))?;
    Ok(ipv6
        .iter()
        .any(|row| u16::from_be(row.dwLocalPort as u16) == port))
}
#[cfg(not(windows))]
pub(super) fn listening(_port: u16) -> Result<bool> {
    Err("Live smoke TCP observation is Windows-only".to_owned())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tcp_table_bounds_reject_overflow_and_truncation() {
        assert_eq!(row_extent(2, 4, 24, 52).unwrap(), 52);
        assert!(row_extent(3, 4, 24, 52).is_err());
        assert!(row_extent(usize::MAX, 4, 24, 52).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn tcp_observer_detects_only_the_owned_fixture_listener() {
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(listening(port).unwrap());
        drop(listener);
        assert!(!listening(port).unwrap());
    }
}
