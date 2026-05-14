use std::io;
use std::time::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Terminal,
};

// --- 1. Konsep OOP: Objek Sensor (Poin 6.3) ---
#[allow(dead_code)] // Menghilangkan warning "unused field" agar tab Problems bersih
struct Sensor {
    nilai_raw: f32,
}

// --- 2. Konsep OOP: Objek Monitoring & Komputasi Numerik (Poin 6.3 & 6.4) ---
struct MonitoringSystem {
    riwayat: Vec<f32>,
    kapasitas_buffer: usize,
}

impl MonitoringSystem {
    fn new(kapasitas: usize) -> Self {
        Self {
            riwayat: Vec::new(),
            kapasitas_buffer: kapasitas,
        }
    }

    // Komputasi Numerik: Simple Moving Average (SMA)
    fn update_dan_hitung_rata_rata(&mut self, data_baru: f32) -> f32 {
        self.riwayat.push(data_baru);
        if self.riwayat.len() > self.kapasitas_buffer {
            self.riwayat.remove(0);
        }
        let total: f32 = self.riwayat.iter().sum();
        total / self.riwayat.len() as f32
    }
}

fn main() -> Result<(), io::Error> {
    // Persiapan Terminal: Mengaktifkan Raw Mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Inisialisasi Data Proyek
    let mut system = MonitoringSystem::new(5); 
    let mut input_simulasi: f32 = 400.0; // Mulai dari angka ideal
    let mut rata_rata_filter: f32 = 400.0;

    // Loop Utama Dashboard
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), 
                    Constraint::Length(3), 
                    Constraint::Min(5),    
                    Constraint::Length(3), 
                ].as_ref())
                .split(f.size());

            // 1. Judul Dashboard
            let title = Paragraph::new("DASHBOARD MONITORING KUALITAS UDARA (ETS ALPROG - RUST)")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // 2. Logika Warna & Gauge Berdasarkan Range Baru
            let gauge_color = if rata_rata_filter > 1000.0 {
                Color::Red // Tercemar
            } else if rata_rata_filter < 400.0 {
                Color::Blue // Sangat Bagus
            } else {
                Color::Green // Ideal
            };

            let gauge = Gauge::default()
                .block(Block::default().title("Konsentrasi Gas (ppm)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(gauge_color))
                .percent(((rata_rata_filter / 1200.0) * 100.0).min(100.0) as u16)
                .label(format!("{:.2} ppm", rata_rata_filter));
            f.render_widget(gauge, chunks[1]);

            // 3. Log Status Berdasarkan Range
            let (kondisi, status_sistem, warna_status) = if rata_rata_filter < 400.0 {
                ("SANGAT BAGUS (Udara Pegunungan/Oksigen Murni)", "SISTEM: Aman - Standby", Color::Blue)
            } else if rata_rata_filter <= 1000.0 {
                ("IDEAL (Udara Bersih)", "SISTEM: Aman - Standby", Color::Green)
            } else {
                ("TERCEMAR (Udara Kotor/Bahaya)", "SISTEM: BAHAYA - Ventilasi AKTIF!", Color::Red)
            };

            // Inisialisasi struct Sensor di sini agar poin OOP tetap terpenuhi
            let _sensor_obj = Sensor { nilai_raw: input_simulasi };

            let log_text = format!(
                "Pembacaan Sensor (Raw): {:.2} ppm\nHasil Filter (SMA): {:.2} ppm\nKondisi Udara : {}\n\n{}",
                _sensor_obj.nilai_raw, rata_rata_filter, kondisi, status_sistem
            );
            
            let log_widget = Paragraph::new(log_text)
                .style(Style::default().fg(warna_status))
                .block(Block::default().title("Log Sistem Kontrol").borders(Borders::ALL))
                .wrap(Wrap { trim: true });
            f.render_widget(log_widget, chunks[2]);

            // 4. Navigasi
            let nav = Paragraph::new("PANAH ATAS (+50) | PANAH BAWAH (-50) | TEKAN 'Q' UNTUK KELUAR")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(nav, chunks[3]);
        })?;

        // Penanganan Input Interaktif
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        input_simulasi = (input_simulasi + 50.0_f32).min(1200.0_f32);
                        rata_rata_filter = system.update_dan_hitung_rata_rata(input_simulasi);
                    },
                    KeyCode::Down => {
                        input_simulasi = (input_simulasi - 50.0_f32).max(0.0_f32);
                        rata_rata_filter = system.update_dan_hitung_rata_rata(input_simulasi);
                    },
                    _ => {}
                }
            }
        }
    }

    // Penutupan Terminal: Mengembalikan ke kondisi semula
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}