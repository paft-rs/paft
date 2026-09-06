use chrono::NaiveDate;
use paft_decimal::parse_decimal;
use paft_fundamentals::{EsgContext, EsgInvolvement, EsgScores, EsgSummary, FundamentalsError};
use serde_json::{Value, json};

fn populated() -> EsgContext {
    let mut context = EsgContext::new(" vendor:Methodology ").unwrap();
    context
        .set_methodology_reference(Some(" reference document ".into()))
        .unwrap();
    context
        .set_methodology_version(Some(" v2 ".into()))
        .unwrap();
    context
        .set_comparison_group(Some(" Industry peers ".into()))
        .unwrap();
    context.set_assessment_date(NaiveDate::from_ymd_opt(2025, 7, 18));
    context
}

#[test]
fn scheme_identity_and_optional_provenance_are_validated_on_every_ingestion_path() {
    let scheme_only: EsgContext =
        serde_json::from_value(json!({"scheme_id":"  vendor:Methodology  "})).unwrap();
    assert_eq!(scheme_only.scheme_id(), "vendor:Methodology");
    assert!(scheme_only.methodology_reference().is_none());
    assert!(scheme_only.methodology_version().is_none());
    assert!(scheme_only.assessment_date().is_none());
    assert!(scheme_only.comparison_group().is_none());
    assert_ne!(scheme_only, EsgContext::new("vendor:methodology").unwrap());
    for value in [
        "",
        " \t\n",
        "vendor",
        ":method",
        "vendor:",
        "vendor:has space",
        "vendor :method",
        "ven\ndor:method",
        "vendor:\u{2003}method",
    ] {
        assert!(matches!(
            EsgContext::new(value),
            Err(FundamentalsError::InvalidEsgContext {
                field: "scheme_id",
                ..
            })
        ));
        assert!(serde_json::from_value::<EsgContext>(json!({"scheme_id":value})).is_err());
    }
    assert!(
        serde_json::from_value::<EsgContext>(json!({"scheme_id":"v:m", "scale":"0..100"})).is_err()
    );
    assert!(serde_json::from_value::<EsgContext>(json!({})).is_err());
    for field in [
        "methodology_reference",
        "methodology_version",
        "comparison_group",
    ] {
        for value in [
            json!(null),
            json!(" legitimate text "),
            json!(""),
            json!(" \t "),
        ] {
            let wire = json!({"scheme_id":"vendor:method", field:value});
            assert_eq!(
                serde_json::from_value::<EsgContext>(wire).is_ok(),
                value.is_null() || value == " legitimate text "
            );
        }
    }
    let context = populated();
    assert_eq!(context.methodology_reference(), Some("reference document"));
    assert_eq!(context.methodology_version(), Some("v2"));
    assert_eq!(context.comparison_group(), Some("Industry peers"));
    assert_eq!(
        context.assessment_date(),
        NaiveDate::from_ymd_opt(2025, 7, 18)
    );
    assert_eq!(
        serde_json::from_value::<EsgContext>(serde_json::to_value(&context).unwrap()).unwrap(),
        context
    );
}

#[test]
fn unsuccessful_mutation_leaves_the_complete_context_unchanged() {
    let mut context = populated();
    let before = context.clone();
    assert!(context.set_scheme_id("provider only").is_err());
    assert_eq!(context, before);
    assert!(context.set_methodology_reference(Some(" ".into())).is_err());
    assert_eq!(context, before);
    assert!(
        context
            .set_methodology_version(Some(String::new()))
            .is_err()
    );
    assert_eq!(context, before);
    assert!(context.set_comparison_group(Some("\t".into())).is_err());
    assert_eq!(context, before);
    context.set_scheme_id(" Other:Scheme ").unwrap();
    context.set_methodology_reference(None).unwrap();
    context.set_methodology_version(None).unwrap();
    context.set_comparison_group(None).unwrap();
    context.set_assessment_date(None);
    assert_eq!(context, EsgContext::new("Other:Scheme").unwrap());
}

#[test]
fn summary_requires_one_context_and_components_retain_standalone_serde() {
    let scores = EsgScores {
        environmental: Some(59.into()),
        social: None,
        governance: Some(60.into()),
    };
    let involvement = EsgInvolvement {
        category: "test_exposure".into(),
        score: Some(parse_decimal("0.25").unwrap()),
    };
    assert_eq!(
        serde_json::from_value::<EsgScores>(serde_json::to_value(&scores).unwrap()).unwrap(),
        scores
    );
    assert_eq!(
        serde_json::from_value::<EsgInvolvement>(serde_json::to_value(&involvement).unwrap())
            .unwrap(),
        involvement
    );
    let mut report = EsgSummary::new(populated());
    report.scores = Some(scores);
    report.involvement = vec![involvement.clone(), involvement];
    let mut wire = serde_json::to_value(&report).unwrap();
    assert_eq!(wire.to_string().matches("scheme_id").count(), 1);
    assert_eq!(
        serde_json::from_value::<EsgSummary>(wire.clone()).unwrap(),
        report
    );
    wire.as_object_mut().unwrap().remove("context");
    assert!(serde_json::from_value::<EsgSummary>(wire).is_err());
    assert!(
        serde_json::from_value::<Option<EsgSummary>>(Value::Null)
            .unwrap()
            .is_none()
    );
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/esg_reports.json")).unwrap()
}

// Test-only mapping. CSA defines its governance component to include economic
// criteria; retaining the scheme is essential to interpreting that value.
fn csa_report() -> EsgSummary {
    let fixture = fixture();
    let raw = &fixture["spglobal"];
    let mut context = EsgContext::new("spglobal:CSA").unwrap();
    context
        .set_methodology_reference(Some(raw["source"].as_str().unwrap().into()))
        .unwrap();
    context
        .set_comparison_group(Some(raw["comparison_group"].as_str().unwrap().into()))
        .unwrap();
    // The source provides a benchmarking date, not an assessment date/version.
    let mut report = EsgSummary::new(context);
    let score =
        |name: &str| Some(parse_decimal(raw["dimensions"][name].as_str().unwrap()).unwrap());
    report.scores = Some(EsgScores {
        environmental: score("Environmental Dimension"),
        social: score("Social Dimension"),
        governance: score("Governance & Economic Dimension"),
    });
    report
}

fn risk_dimensions(raw: &Value) -> Result<EsgScores, &'static str> {
    if raw.get("ESG Risk Rating").is_some() {
        return Err("overall unmanaged risk is not a dimension score");
    }
    Err("this reduced fixture supplies no supported dimension scores")
}

#[test]
fn attributed_provider_mappings_preserve_methodology_and_reject_false_equivalence() {
    let report = csa_report();
    let wire = serde_json::to_value(&report).unwrap();
    assert_eq!(wire["context"]["scheme_id"], "spglobal:CSA");
    assert_eq!(
        wire["scores"],
        json!({"environmental":"59", "social":"58", "governance":"60"})
    );
    assert!(report.context.assessment_date().is_none());
    assert_eq!(serde_json::from_value::<EsgSummary>(wire).unwrap(), report);

    let raw = &fixture()["sustainalytics"];
    assert_eq!(
        parse_decimal(raw["ESG Risk Rating"].as_str().unwrap()).unwrap(),
        parse_decimal("18.3").unwrap()
    );
    let context = EsgContext::new("sustainalytics:ESG_RISK_RATINGS").unwrap();
    assert_ne!(context.scheme_id(), report.context.scheme_id());
    assert_eq!(
        serde_json::from_value::<EsgContext>(serde_json::to_value(&context).unwrap()).unwrap(),
        context
    );
    assert_eq!(
        risk_dimensions(raw),
        Err("overall unmanaged risk is not a dimension score")
    );
}

#[cfg(feature = "dataframe")]
#[test]
fn shared_context_survives_dataframe_exports_with_retained_component_traits() {
    use paft_decimal::Decimal;
    use paft_utils::dataframe::{Columnar, ToDataFrame, ToDataFrameVec};
    use polars::prelude::{AnyValue, DataType};
    let report = csa_report();
    let rows = [
        report.clone(),
        EsgSummary::new(EsgContext::new("different:Scheme").unwrap()),
    ];
    let refs: Vec<_> = rows.iter().collect();
    for df in [
        rows.to_dataframe().unwrap(),
        EsgSummary::columnar_from_refs(&refs).unwrap(),
        report.to_dataframe().unwrap(),
    ] {
        assert_eq!(df.schema(), EsgSummary::empty_dataframe().unwrap().schema());
        assert_eq!(
            df.column("context.scheme_id")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            Some("spglobal:CSA")
        );
        assert_eq!(
            df.column("context.assessment_date").unwrap().dtype(),
            &DataType::Date
        );
        assert_eq!(
            df.column("scores.environmental").unwrap().get(0).unwrap(),
            AnyValue::Decimal(590_000_000_000, 38, 10)
        );
        assert_eq!(
            df.get_column_names()
                .iter()
                .filter(|name| name.as_str().ends_with("scheme_id"))
                .count(),
            1
        );
        if df.height() == 2 {
            assert_eq!(
                df.column("context.scheme_id")
                    .unwrap()
                    .str()
                    .unwrap()
                    .get(1),
                Some("different:Scheme")
            );
            assert_eq!(
                df.column("scores.environmental").unwrap().get(1).unwrap(),
                AnyValue::Null
            );
        }
    }
    report.scores.unwrap().to_dataframe().unwrap();
    EsgInvolvement {
        category: "test".into(),
        score: Some(Decimal::ZERO),
    }
    .to_dataframe()
    .unwrap();
    populated().to_dataframe().unwrap();
}
