use sfstudio_core::animation::{AnimationSymbol, Effect, Scope};
use sfstudio_core::settings::ThemeMode;
use sfstudio_core::vector::{interpolate_points, vector_catalog, Geometry};
use sfstudio_core::{catalog, Symbol};
use slint::{ComponentHandle, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

slint::include_modules!();

const RESULT_LIMIT: usize = 96;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;
    let selected_name = Rc::new(RefCell::new("0.circle.fill".to_owned()));
    let selected_category = Rc::new(RefCell::new(None::<String>));
    let query = Rc::new(RefCell::new(String::new()));

    ui.set_categories(
        Rc::new(VecModel::from(
            catalog()
                .category_counts()
                .into_iter()
                .take(16)
                .map(|(name, count)| CategoryRow {
                    name: name.into(),
                    count: count.min(i32::MAX as usize) as i32,
                })
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    refresh(&ui, "", None, &selected_name.borrow());

    let weak = ui.as_weak();
    let category = Rc::clone(&selected_category);
    let selected = Rc::clone(&selected_name);
    let query_state = Rc::clone(&query);
    ui.on_search_changed(move |value| {
        *query_state.borrow_mut() = value.to_string();
        if let Some(ui) = weak.upgrade() {
            refresh(
                &ui,
                &query_state.borrow(),
                category.borrow().as_deref(),
                &selected.borrow(),
            );
        }
    });

    let weak = ui.as_weak();
    let category = Rc::clone(&selected_category);
    let selected = Rc::clone(&selected_name);
    let query_state = Rc::clone(&query);
    ui.on_category_selected(move |name| {
        *category.borrow_mut() = (!name.is_empty()).then(|| name.to_string());
        if let Some(ui) = weak.upgrade() {
            refresh(
                &ui,
                &query_state.borrow(),
                category.borrow().as_deref(),
                &selected.borrow(),
            );
        }
    });

    let weak = ui.as_weak();
    let category = Rc::clone(&selected_category);
    let selected = Rc::clone(&selected_name);
    let query_state = Rc::clone(&query);
    ui.on_symbol_selected(move |name| {
        *selected.borrow_mut() = name.to_string();
        if let Some(ui) = weak.upgrade() {
            refresh(
                &ui,
                &query_state.borrow(),
                category.borrow().as_deref(),
                &selected.borrow(),
            );
        }
    });

    let weak = ui.as_weak();
    ui.on_theme_selected(move |id| {
        if let Some(mode) = ThemeMode::from_id(id) {
            if let Some(ui) = weak.upgrade() {
                ui.set_theme_mode(match mode {
                    ThemeMode::System => 0,
                    ThemeMode::Light => 1,
                    ThemeMode::Dark => 2,
                });
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_settings_selected(move |visible| {
        if let Some(ui) = weak.upgrade() {
            ui.set_settings_screen(visible);
        }
    });

    let animator = Rc::new(RefCell::new(None::<(Instant, AnimationSymbol)>));
    let animation_timer = Rc::new(Timer::default());
    let weak = ui.as_weak();
    let selected = Rc::clone(&selected_name);
    let animator_for_start = Rc::clone(&animator);
    let timer_for_start = animation_timer.clone();
    ui.on_play_effect(move || {
        let Ok(Some(glyph)) =
            vector_catalog().and_then(|vectors| vectors.glyph(&selected.borrow()))
        else {
            return;
        };
        *animator_for_start.borrow_mut() = Some((
            Instant::now(),
            AnimationSymbol::from_glyph(&glyph, &glyph.geometry),
        ));
        timer_for_start.start(TimerMode::Repeated, Duration::from_millis(16), {
            let weak = weak.clone();
            let animator = Rc::clone(&animator_for_start);
            let timer = Rc::clone(&timer_for_start);
            move || {
                let Some(ui) = weak.upgrade() else {
                    timer.stop();
                    return;
                };
                let frame = {
                    let animation = animator.borrow();
                    let Some((started, symbol)) = animation.as_ref() else {
                        timer.stop();
                        return;
                    };
                    let time = started.elapsed().as_secs_f32();
                    if time
                        > symbol.duration(Effect::Bounce {
                            up: true,
                            scope: Scope::WholeSymbol,
                        })
                    {
                        None
                    } else {
                        symbol.apply(
                            Effect::Bounce {
                                up: true,
                                scope: Scope::WholeSymbol,
                            },
                            time,
                            0,
                            false,
                        )
                    }
                };
                if let Some(frame) = frame {
                    ui.set_animation_scale(frame.scale);
                    ui.set_animation_dy(frame.dy);
                    ui.set_animation_rotation(frame.rotation);
                    ui.set_animation_alpha(frame.alpha);
                } else {
                    ui.set_animation_scale(1.0);
                    ui.set_animation_dy(0.0);
                    ui.set_animation_rotation(0.0);
                    ui.set_animation_alpha(1.0);
                    *animator.borrow_mut() = None;
                    timer.stop();
                }
            }
        });
    });

    ui.run()?;
    Ok(())
}

fn refresh(ui: &MainWindow, query: &str, category: Option<&str>, selected_name: &str) {
    let symbols = catalog().search_filtered(query, category, None, RESULT_LIMIT);
    let total = if query.trim().is_empty() && category.is_none() {
        catalog().all().len()
    } else {
        symbols.len()
    };
    ui.set_result_count(format!("{total} symbols").into());
    ui.set_symbols(
        Rc::new(VecModel::from(
            symbols
                .into_iter()
                .filter_map(|symbol| symbol_row(symbol, selected_name))
                .collect::<Vec<_>>(),
        ))
        .into(),
    );
    ui.set_selected_name(selected_name.into());
    ui.set_selected_path(glyph_path(selected_name).unwrap_or_default().into());
}

fn symbol_row(symbol: &Symbol, selected_name: &str) -> Option<SymbolRow> {
    Some(SymbolRow {
        name: symbol.apple_name.as_str().into(),
        path: glyph_path(&symbol.apple_name)?.into(),
        selected: symbol.apple_name == selected_name,
    })
}

fn glyph_path(name: &str) -> Option<String> {
    let glyph = vector_catalog().ok()?.glyph(name).ok()??;
    svg_path(&glyph.geometry).ok()
}

fn svg_path(geometry: &Geometry) -> Result<String, String> {
    let points = interpolate_points(geometry, 400.0);
    if points.iter().any(|point| !point.is_finite()) {
        return Err("symbol geometry contains non-finite coordinates".to_owned());
    }
    let mut output = String::new();
    let mut index = 0;
    for op in &geometry.ops {
        let coordinate_count = match op {
            0 | 1 => 2,
            2 => 6,
            3 => 0,
            _ => return Err(format!("unknown vector operation {op}")),
        };
        let coordinates = points
            .get(index..index + coordinate_count)
            .ok_or_else(|| "vector operation exceeds coordinate data".to_owned())?;
        match op {
            0 => output.push_str(&format!(
                "M{} {} ",
                svg_coordinate(coordinates[0]),
                svg_coordinate(coordinates[1])
            )),
            1 => output.push_str(&format!(
                "L{} {} ",
                svg_coordinate(coordinates[0]),
                svg_coordinate(coordinates[1])
            )),
            2 => output.push_str(&format!(
                "C{} {} {} {} {} {} ",
                svg_coordinate(coordinates[0]),
                svg_coordinate(coordinates[1]),
                svg_coordinate(coordinates[2]),
                svg_coordinate(coordinates[3]),
                svg_coordinate(coordinates[4]),
                svg_coordinate(coordinates[5])
            )),
            3 => output.push_str("Z "),
            _ => unreachable!(),
        }
        index += coordinate_count;
    }
    if index != points.len() {
        return Err("vector geometry contains unused coordinates".to_owned());
    }
    Ok(output)
}

fn svg_coordinate(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::svg_path;
    use sfstudio_core::vector::Geometry;

    #[test]
    fn svg_path_serializes_moves_lines_cubics_and_closes() {
        let geometry = Geometry {
            ops: vec![0, 1, 2, 3],
            regular: vec![0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            light: vec![0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
            heavy: vec![0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
        };
        assert_eq!(
            svg_path(&geometry).unwrap(),
            "M0 0 L1 -1 C2 -3 4 -5 6 -7 Z "
        );
    }

    #[test]
    fn svg_path_rejects_mismatched_operation_data() {
        let geometry = Geometry {
            ops: vec![2],
            regular: vec![0.0; 2],
            light: vec![0.0; 2],
            heavy: vec![0.0; 2],
        };
        assert!(svg_path(&geometry).is_err());
    }
}
