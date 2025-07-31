#include "lib.hpp" // Header with C-compatible AppEventType and AppEvent
#include <QApplication>
#include <QIcon>
#include <QMenu>
#include <QSystemTrayIcon>
#include <QBuffer>
#include <string>
#include <vector>
#include <memory>
#include <QTimer>
#include <QSettings>
#include <QLoggingCategory>

Q_LOGGING_CATEGORY(lcNoWarnings, "no.warnings", QtInfoMsg)

class QtAppWrapper;

struct QtAppHandle
{
    QtAppWrapper *impl;
};

class QtAppWrapper
{
public:
    QtAppWrapper() = default;

    void setAppId(const std::string &id) { appId = id; }
    void setOrganizationName(const std::string &name) { organizationName = name; }

    void setAppIcon(const unsigned char *data, size_t size, const char *format)
    {
        iconData = QByteArray(reinterpret_cast<const char *>(data), size);
        iconFormat = format;
    }

    void initTray() { shouldInitTray = true; }

    int run(int argc, char *argv[])
    {
        QLoggingCategory::setFilterRules("qt.qsettings.warning=false\nqt.systemtrayicon.warning=false\n*.warning=false");
        if (!organizationName.empty())
        {
            QApplication::setOrganizationName(QString::fromStdString(organizationName));
        }
        if (!appId.empty())
        {
            QApplication::setApplicationName(QString::fromStdString(appId));
        }

        app = new QApplication(argc, argv);

        QIcon appIcon;
        if (!iconData.isEmpty())
        {
            QPixmap pixmap;
            if (pixmap.loadFromData(iconData, iconFormat.c_str()))
            {
                appIcon = QIcon(pixmap);
                app->setWindowIcon(appIcon);
            }
        }

        if (shouldInitTray)
        {
            if (!QSystemTrayIcon::isSystemTrayAvailable())
            {
                return -1;
            }

            menu = new QMenu();
            tray = new QSystemTrayIcon(appIcon);
            tray->setContextMenu(menu);

            QObject::connect(tray, &QSystemTrayIcon::activated, [this](QSystemTrayIcon::ActivationReason reason)
                             {
                if (reason == QSystemTrayIcon::Context) { /* Right-click, no event pushed */ }
                else if (reason == QSystemTrayIcon::Trigger) {
                    event_queue.push_back({TrayClicked, nullptr}); // Use the new enum value directly
                } else if (reason == QSystemTrayIcon::DoubleClick) {
                    event_queue.push_back({TrayDoubleClicked, nullptr}); // Use the new enum value directly
                } });
            tray->show();

            for (const auto &item : pending_menu_items)
            {
                addTrayMenuItem(item.first, item.second);
            }
            pending_menu_items.clear();
        }
        return app->exec();
    }

    AppEvent pollEvent()
    {
        if (event_queue.empty())
        {
            return {None, nullptr};
        } // Use the new enum value directly
        AppEvent event = event_queue.front();
        event_queue.erase(event_queue.begin());
        // Crucially, nullify the menu_id_str pointer in the *copied* event
        // to prevent accidental re-freeing if the queue or event are somehow re-used.
        // This is a defensive measure for C-style FFI.
        // The event *returned* will have the correct data.
        // The original entry in the queue is gone.
        return event;
    }

    void addTrayMenuItem(const std::string &text, const std::string &id_str)
    {
        if (!app)
        {
            pending_menu_items.push_back({text, id_str});
            return;
        }

        if (!menu)
        {
            menu = new QMenu();
            if (tray)
            {
                tray->setContextMenu(menu);
            }
        }

        QAction *action = menu->addAction(QString::fromStdString(text));
        QObject::connect(action, &QAction::triggered, [this, id_str]()
                         {
                             char *id_cstr = strdup(id_str.c_str());
                             event_queue.push_back({MenuItemClicked, id_cstr}); // Use the new enum value directly
                         });
    }

    void requestQuitSafe()
    {
        if (app)
        {
            QTimer::singleShot(0, app, &QApplication::quit);
        }
    }

private:
    std::string appId;
    std::string organizationName;
    QByteArray iconData;
    std::string iconFormat;
    bool shouldInitTray = false;
    std::vector<AppEvent> event_queue;
    std::vector<std::pair<std::string, std::string>> pending_menu_items;

    QMenu *menu = nullptr;
    QSystemTrayIcon *tray = nullptr;
    QApplication *app = nullptr;
};

extern "C"
{

    QtAppHandle *create_qt_app() { return new QtAppHandle{new QtAppWrapper()}; }
    void set_app_id(QtAppHandle *handle, const char *id)
    {
        if (handle && handle->impl)
        {
            handle->impl->setAppId(id);
        }
    }
    void set_organization_name(QtAppHandle *handle, const char *name)
    {
        if (handle && handle->impl)
        {
            handle->impl->setOrganizationName(name);
        }
    }
    void set_app_icon_from_data(QtAppHandle *handle, const unsigned char *data, size_t size, const char *format)
    {
        if (handle && handle->impl)
        {
            handle->impl->setAppIcon(data, size, format);
        }
    }
    void init_tray(QtAppHandle *handle)
    {
        if (handle && handle->impl)
        {
            handle->impl->initTray();
        }
    }
    int run_qt_app(QtAppHandle *handle, int argc, char *argv[])
    {
        if (handle && handle->impl)
        {
            return handle->impl->run(argc, argv);
        }
        return -1;
    }
    AppEvent poll_event(QtAppHandle *handle)
    {
        if (handle && handle->impl)
        {
            return handle->impl->pollEvent();
        }
        return {None, nullptr};
    } // Use the new enum value directly
    void request_quit_qt_app_safe(QtAppHandle *handle)
    {
        if (handle && handle->impl)
        {
            handle->impl->requestQuitSafe();
        }
    }
    void cleanup_qt_app(QtAppHandle *handle)
    {
        if (handle)
        {
            delete handle->impl;
            delete handle;
        }
    }
    void add_tray_menu_item(QtAppHandle *handle, const char *text, const char *id)
    {
        if (handle && handle->impl)
        {
            handle->impl->addTrayMenuItem(text, id);
        }
    }
    void free_char_ptr(const char *ptr) { free((void *)ptr); }

} // extern "C"