# Mini JIRA App

A lightweight desktop application for JIRA worklog management built with Tauri v2, Svelte 5, and Rust. This app runs in the background with system tray integration and daily notification reminders.

## ✨ Features

- **JIRA Integration**: Connect to JIRA using Basic Authentication (email + API token)
- **Worklog Management**: Create and submit worklog entries with flexible time formats
- **Background Operation**: Runs in system tray with daily 5 PM reminders
- **Cross-Platform**: Built with Tauri for Windows, macOS, and Linux
- **Modern UI**: Clean interface built with Svelte 5 and Tailwind CSS
- **Multiple Build Modes**: Debug and release configurations
- **Portable Builds**: NSIS portable installer and MSI package support

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust](https://rustlang.org/) (latest stable)
- [Bun](https://bun.sh/) (package manager)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd mini-jira-app
```

2. Install dependencies:
```bash
bun install
```

3. Run in development mode:
```bash
bun run tauri:dev
```

## 🔧 Build & Deployment

### Development Build
```bash
bun run build:debug
bun run tauri:build:debug
```

### Production Build
```bash
bun run build:release
bun run tauri:build
```

### Portable & MSI Builds
```bash
bun run tauri:build:all
```

### No Bundle Build (Executable Only)
```bash
bun run tauri:build:no-bundle
```

## 📱 Usage

### Initial Setup
1. Launch the application
2. Enter your JIRA credentials:
   - **Base URL**: Your JIRA instance URL (e.g., `https://company.atlassian.net`)
   - **Email**: Your JIRA account email
   - **API Token**: Generate from JIRA Account Settings → Security → API tokens

### Creating Worklogs
1. Select an issue from your assigned issues
2. Choose the work date
3. Enter time spent (supports formats like `2h`, `30m`, `1d`)
4. Add a description of work done
5. Submit the worklog

### Background Features
- **System Tray**: App minimizes to system tray instead of closing
- **Daily Reminders**: Automatic notifications at 5 PM
- **Persistent Connection**: Stays connected to JIRA in the background

## 🏗️ Technical Architecture

### Frontend
- **Svelte 5**: Modern reactive framework with runes
- **TypeScript**: Type-safe development
- **Tailwind CSS**: Utility-first styling
- **Vite**: Fast build tool and dev server

### Backend
- **Rust**: High-performance system programming
- **Tauri v2**: Cross-platform desktop app framework
- **reqwest**: HTTP client for JIRA API integration
- **tokio**: Async runtime for background tasks

### JIRA Integration
- **REST API v3**: Official JIRA REST API
- **Basic Authentication**: Email + API token
- **SSL Support**: Handles corporate SSL certificates

## 🔒 Security Features

- **Local Storage**: Credentials stored locally (not transmitted)
- **SSL Bypass**: Option for corporate environments with self-signed certificates
- **No Data Persistence**: No sensitive data stored on external servers

## 🎛️ Configuration

### Debug Mode
When running in development mode, additional controls are available:
- Manual background mode toggle
- Test notification triggers
- Enhanced logging

### Environment Variables
The app automatically detects build mode:
- Development: `import.meta.env.DEV === true`
- Production: `import.meta.env.MODE === 'production'`

## 📁 Project Structure

```
mini-jira-app/
├── src/                    # Svelte frontend
│   ├── routes/
│   │   └── +page.svelte   # Main application UI
│   └── app.html           # HTML template
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # Tauri commands & tray
│   │   ├── jira_api.rs    # JIRA HTTP client
│   │   └── jira_types.rs  # Data structures
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── static/                # Static assets
└── package.json           # Node.js dependencies
```

## 🛠️ Development

### Available Scripts

- `bun run dev` - Start Vite dev server
- `bun run tauri:dev` - Start Tauri in development mode
- `bun run build` - Build frontend for production
- `bun run tauri:build` - Build complete desktop app
- `bun run check` - Run TypeScript and Svelte checks

### Adding Features

1. **Frontend Changes**: Edit files in `src/` directory
2. **Backend Logic**: Modify Rust files in `src-tauri/src/`
3. **JIRA API**: Extend `jira_api.rs` for new endpoints
4. **UI Components**: Add new Svelte components in `src/routes/`

## 🐛 Troubleshooting

### Common Issues

**Connection Failed**
- Verify JIRA URL is correct and accessible
- Check API token is valid and not expired
- Ensure email address matches JIRA account

**SSL Certificate Errors**
- App includes SSL bypass for corporate environments
- Verify network connectivity to JIRA instance

**System Tray Not Working**
- Ensure system supports notification area
- Check OS permissions for background apps

### Debug Mode
Run with debug flag to see detailed logs:
```bash
bun run tauri:build:debug
```

## 📋 Requirements

### System Requirements
- **Windows**: Windows 10/11 (64-bit)
- **macOS**: macOS 10.15+ (Intel/Apple Silicon)
- **Linux**: GTK3 compatible distributions

### JIRA Requirements
- JIRA Cloud or Server/Data Center
- Valid user account with worklog permissions
- API token for authentication

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🆘 Support

For issues and questions:
- Check existing GitHub Issues
- Create new issue with detailed description
- Include system information and error logs

---

Built with 🔥 and ☕ using Tauri, Svelte, and Rust
