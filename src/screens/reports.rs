use iced::widget::canvas::Canvas;
use iced::widget::{column, container, text};
use iced::{Element, Fill};

use crate::message::Message;
use crate::reports::employee_type_pie::EmployeeTypePieReport;
use crate::reports::heatmap_site_employee::SiteEmployeeHeatmapReport;
use crate::reports::it_lead_time::ItLeadTimeReport;
use crate::reports::radar_lead_time::LeadTimeRadarReport;
use crate::reports::sla_breach_circle::SlaBreachCircleReport;
use crate::reports::state_counts_bar::StateCountsBarReport;

pub fn view<'a>(
    _collapsed: bool,
    loading: bool,
    error: Option<&'a str>,
    points: &[(f32, f32)],
    state_loading: bool,
    state_error: Option<&'a str>,
    state_points: &[(String, f32)],
    employee_loading: bool,
    employee_error: Option<&'a str>,
    employee_points: &[(String, f32)],
    heatmap_loading: bool,
    heatmap_error: Option<&'a str>,
    heatmap_grid: Option<&(Vec<String>, Vec<String>, Vec<Vec<f32>>)>,
    radar_loading: bool,
    radar_error: Option<&'a str>,
    radar_metrics: &[(String, f32)],
    breach_loading: bool,
    breach_error: Option<&'a str>,
    breach_rate: Option<(f32, f32)>,
) -> Element<'a, Message> {
    let chart = ItLeadTimeReport::chart(points);
    let bar_chart = StateCountsBarReport::chart(state_points);
    let pie_chart = EmployeeTypePieReport::chart(employee_points);
    let heatmap_chart = heatmap_grid.map(|grid| {
        SiteEmployeeHeatmapReport::chart(crate::charts::HeatmapGrid {
            x_labels: grid.0.clone(),
            y_labels: grid.1.clone(),
            values: grid.2.clone(),
        })
    });
    let radar_chart = LeadTimeRadarReport::chart(radar_metrics);
    let circle_chart = breach_rate.map(|(breaches, total)| {
        SlaBreachCircleReport::chart(breaches, total)
    });

    let mut content = column![text("Reports").size(28)].spacing(24);

    if loading {
        content = content.push(text("Loading chart data...").size(14));
    } else if let Some(message) = error {
        content = content.push(text(message).size(14));
    } else if points.is_empty() {
        content = content.push(text("No data available yet.").size(14));
    }

    content = content.push(chart_section(
        ItLeadTimeReport::title(),
        ItLeadTimeReport::subtitle(),
        Canvas::new(chart).width(Fill).height(260),
        loading,
        error,
        points.is_empty(),
    ));

    content = content.push(chart_section(
        StateCountsBarReport::title(),
        StateCountsBarReport::subtitle(),
        Canvas::new(bar_chart).width(Fill).height(260),
        state_loading,
        state_error,
        state_points.is_empty(),
    ));

    content = content.push(chart_section(
        EmployeeTypePieReport::title(),
        EmployeeTypePieReport::subtitle(),
        Canvas::new(pie_chart).width(Fill).height(260),
        employee_loading,
        employee_error,
        employee_points.is_empty(),
    ));

    if let Some(heatmap_chart) = heatmap_chart {
        content = content.push(chart_section(
            SiteEmployeeHeatmapReport::title(),
            SiteEmployeeHeatmapReport::subtitle(),
            Canvas::new(heatmap_chart).width(Fill).height(320),
            heatmap_loading,
            heatmap_error,
            false,
        ));
    }

    content = content.push(chart_section(
        LeadTimeRadarReport::title(),
        LeadTimeRadarReport::subtitle(),
        Canvas::new(radar_chart).width(Fill).height(280),
        radar_loading,
        radar_error,
        radar_metrics.is_empty(),
    ));

    if let Some(circle_chart) = circle_chart {
        content = content.push(chart_section(
            SlaBreachCircleReport::title(),
            SlaBreachCircleReport::subtitle(),
            Canvas::new(circle_chart).width(Fill).height(240),
            breach_loading,
            breach_error,
            false,
        ));
    }

    container(content).padding(24).into()
}

fn chart_section<'a>(
    title: &'static str,
    subtitle: &'static str,
    chart: impl Into<Element<'a, Message>>,
    loading: bool,
    error: Option<&'a str>,
    empty: bool,
) -> Element<'a, Message> {
    let mut section = column![text(title).size(18), text(subtitle).size(14)]
        .spacing(8)
        .push(chart);

    if loading {
        section = section.push(text("Loading data...").size(14));
    } else if let Some(message) = error {
        section = section.push(text(message).size(14));
    } else if empty {
        section = section.push(text("No data available yet.").size(14));
    }

    container(section)
        .padding(16)
        .style(|theme| iced::widget::container::bordered_box(theme))
        .into()
}
