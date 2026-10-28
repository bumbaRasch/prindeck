use super::CupsJob;
use super::Sides;
use super::lp::parse_jobs;

const FIXTURE: &str = include_str!("../../../fixtures/print/lpstat-not-completed.txt");

#[test]
fn parses_fixture_rows() {
    let jobs = parse_jobs("Brother_MFC_L2710DW_series", FIXTURE);

    assert_eq!(
        jobs,
        vec![CupsJob {
            id: "Brother_MFC_L2710DW_series-4".to_string(),
            owner: "user".to_string(),
            size_kb: 1024,
        }]
    );
}

#[test]
fn non_matching_queue_is_dropped() {
    let output = "\
Office_Printer-7 root 2048 Tue Oct 27 16:10:28 2026
Brother_MFC_L2710DW_series_backup-9 root 512 Tue Oct 27 16:10:28 2026
";

    let jobs = parse_jobs("Brother_MFC_L2710DW_series", output);

    assert_eq!(jobs, Vec::new());
}

#[test]
fn sides_option_strings() {
    assert_eq!(Sides::OneSided.as_lp_option(), "one-sided");
    assert_eq!(Sides::LongEdge.as_lp_option(), "two-sided-long-edge");
    assert_eq!(Sides::ShortEdge.as_lp_option(), "two-sided-short-edge");
}

#[test]
fn empty_output_yields_empty_vec() {
    assert_eq!(parse_jobs("Brother_MFC_L2710DW_series", ""), Vec::new());
}

#[test]
fn garbage_line_is_skipped_not_fatal() {
    let output = "\
total 0
!!! total garbage !!!
Brother_MFC_L2710DW_series-4 user 1024 Tue Oct 27 16:10:28 2026
Brother_MFC_L2710DW_series-5
Brother_MFC_L2710DW_series-x root notanumber Tue Oct 27 16:10:28 2026
";

    let jobs = parse_jobs("Brother_MFC_L2710DW_series", output);

    assert_eq!(
        jobs,
        vec![CupsJob {
            id: "Brother_MFC_L2710DW_series-4".to_string(),
            owner: "user".to_string(),
            size_kb: 1024,
        }]
    );
}
