use crate::monitoing_report::models::MonitoringReportData;
use chrono::{TimeZone, Utc};
use docx_rs::*;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyDict, PyType};
use pyo3_asyncio::tokio::future_into_py;
use serde_json;
use std::io::Cursor;

#[pyclass]
pub struct WordReport;

#[pymethods]
impl WordReport {
    #[classmethod]
    pub fn generate_word<'py>(
        _cls: &PyType,
        py: Python<'py>,
        data: &PyAny,
    ) -> PyResult<&'py PyAny> {
        let json_str = data.str()?.to_str()?.to_string();

        const CELL_FILL: &str = "7DCEA0";
        const FONT_SIZE: usize = 24;
        const CELL_WIDTH: usize = 4320; // 7.62 см в twips

        future_into_py::<_, PyObject>(py, async move {
            let parsed: MonitoringReportData = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

            let buf = tokio::task::spawn_blocking(move || {
                let mut buf = Vec::new();

                // Получаем текущую дату
                let current_date = Utc::now().format("%Y-%m-%d").to_string();

                // Создаем имя файла по шаблону
                let filename = format!(
                    "Отчет о результатах мониторинга {} {}.docx",
                    parsed.account_name, current_date
                );

                // Функция для создания Arial шрифта
                let arial_fonts = || {
                    RunFonts::new()
                        .ascii("Arial")
                        .east_asia("Arial")
                        .hi_ansi("Arial")
                };

                // Функция для создания стилизованного текста
                let styled_run = |text: &str| {
                    Run::new()
                        .add_text(text)
                        .bold()
                        .size(FONT_SIZE)
                        .fonts(arial_fonts())
                };

                // Функция для создания ячейки с заливкой
                let create_header_cell = |text: &str| {
                    TableCell::new()
                        .shading(Shading::new().fill(CELL_FILL))
                        .width(CELL_WIDTH, WidthType::Dxa)
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(styled_run(text))
                                .align(AlignmentType::Left),
                        )
                };

                // Функция для создания обычной ячейки (с такой же шириной!)
                let create_data_cell = |text: &str| {
                    TableCell::new()
                        .width(CELL_WIDTH, WidthType::Dxa) // Добавляем ширину
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(styled_run(text))
                                .align(AlignmentType::Left),
                        )
                };

                // Данные для таблицы - динамически заполняются из parsed
                let table_data = vec![
                    ("ИНН", parsed.account_inn.as_str()),
                    ("Наименование контрагента", parsed.account_name.as_str()),
                    ("Скоринг", ""), // Нет соответствующего поля в структуре
                    ("Отрасль", parsed.industry.as_str()),
                    ("Сегмент", parsed.business_segment.as_str()),
                    ("Принадлежность к группе", ""), // Нет соответствующего поля в структуре
                ];

                // Создание строк таблицы
                let table_rows: Vec<TableRow> = table_data
                    .into_iter()
                    .map(|(header, value)| {
                        TableRow::new(vec![create_header_cell(header), create_data_cell(value)])
                    })
                    .collect();

                let doc = Docx::new()
                    .page_margin(PageMargin::new().top(851)) // 1.5 см
                    .add_paragraph(
                        Paragraph::new()
                            .add_run(
                                Run::new()
                                    .add_text("Отчет о результатах мониторинга контрагента")
                                    .bold()
                                    .size(FONT_SIZE)
                                    .fonts(arial_fonts()),
                            )
                            .align(AlignmentType::Center),
                    )
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))) // Пустая строка
                    .add_table(Table::new(table_rows).align(TableAlignmentType::Center))
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))) // Пустая строка
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))) // Пустая строка
                    .add_table(
                        Table::new(vec![TableRow::new(vec![
                            create_header_cell("Выявленный риск-сигнал"),
                            create_data_cell(&parsed.signal_name),
                        ])])
                        .align(TableAlignmentType::Center),
                    )
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))) // Пустая строка
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))) // Пустая строка
                    .add_paragraph(
                        Paragraph::new()
                            .add_run(
                                Run::new()
                                    .add_text("Комментарии:")
                                    .bold()
                                    .size(FONT_SIZE)
                                    .fonts(arial_fonts()),
                            )
                            .align(AlignmentType::Left),
                    )
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))); // Пустая строка

                // Добавляем динамические комментарии из parsed.description
                let doc = match &parsed.description {
                    Some(description) => {
                        // Разбиваем описание на строки и добавляем каждую как отдельный параграф
                        let comment_lines: Vec<&str> = description.lines().collect();

                        let mut doc_with_comments = doc;
                        for line in comment_lines {
                            if !line.trim().is_empty() {
                                doc_with_comments = doc_with_comments.add_paragraph(
                                    Paragraph::new()
                                        .add_run(
                                            Run::new()
                                                .add_text(line.trim())
                                                .size(FONT_SIZE - 4) // Чуть меньше шрифт
                                                .fonts(arial_fonts())
                                                .italic(),
                                        )
                                        .align(AlignmentType::Right),
                                );
                            }
                        }
                        doc_with_comments
                    }
                    None => {
                        // Если description нет, показываем стандартные комментарии
                        doc.add_paragraph(
                            Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text("Описание риска")
                                        .size(FONT_SIZE - 4) // Чуть меньше шрифт
                                        .fonts(arial_fonts())
                                        .italic(),
                                )
                                .align(AlignmentType::Right),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text("История и причины возникновения")
                                        .size(FONT_SIZE - 4)
                                        .fonts(arial_fonts())
                                        .italic(),
                                )
                                .align(AlignmentType::Right),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text("Комментарии клиентского менеджера")
                                        .size(FONT_SIZE - 4)
                                        .fonts(arial_fonts())
                                        .italic(),
                                )
                                .align(AlignmentType::Right),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text("Перспективы закрытия риск-сигнала")
                                        .size(FONT_SIZE - 4)
                                        .fonts(arial_fonts())
                                        .italic(),
                                )
                                .align(AlignmentType::Right),
                        )
                        .add_paragraph(
                            Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text("Прочие комментарии")
                                        .size(FONT_SIZE - 4)
                                        .fonts(arial_fonts())
                                        .italic(),
                                )
                                .align(AlignmentType::Right),
                        )
                    }
                };

                // Добавляем 10 пустых строк с помощью цикла
                let mut doc = doc;
                for _ in 0..25 {
                    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))); // Пустая строка
                }

                // Добавляем таблицу сотрудника с текущей датой
                let doc = doc.add_table({
                    let user_rows = vec![
                        // Заголовок таблицы
                        TableRow::new(vec![
                            create_header_cell("Сотрудник"),
                            create_header_cell("Дата составления"),
                        ]),
                        // Одна строка с данными из parsed и текущей датой
                        TableRow::new(vec![
                            create_data_cell(parsed.created_by.as_deref().unwrap_or("Не указан")),
                            create_data_cell(&current_date), // Используем текущую дату
                        ]),
                    ];

                    Table::new(user_rows).align(TableAlignmentType::Center)
                });

                doc.build().pack(Cursor::new(&mut buf)).unwrap();

                // Возвращаем буфер и имя файла как кортеж
                (buf, filename)
            })
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Python::with_gil(|py| {
                let (buffer, filename) = buf;

                // Создаем словарь с данными файла и именем
                let result = PyDict::new(py);
                result.set_item("file_data", PyBytes::new(py, &buffer))?;
                result.set_item("filename", filename)?;

                Ok(result.into())
            })
        })
    }
}
