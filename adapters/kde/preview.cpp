// MIT; copyright 2026 Fjord contributors. Isolated application palette preview.
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QProgressBar>
#include <QPushButton>
#include <QStyle>
#include <QTimer>
#include <QVBoxLayout>
#include <KColorScheme>
#include <KSharedConfig>

int main(int argc,char **argv) {
 QApplication app(argc,argv);
#ifdef Q_OS_LINUX
 // Linux-only preview control: avoid caret-phase noise in deterministic captures.
 QApplication::setCursorFlashTime(0);
#endif
 app.setApplicationName("Fjord isolated KDE preview");
 if(app.arguments().size()!=3)return 2;
 const QString scheme=app.arguments().at(1), output=app.arguments().at(2);
 app.setStyle("Breeze");
 QWidget window;window.setWindowTitle("Fjord isolated native colour preview");window.resize(680,560);
 auto layout=new QVBoxLayout(&window);
 layout->addWidget(new QLabel("Fjord — KDE / Qt Widgets colour proof of concept"));
 layout->addWidget(new QLabel("Opaque surfaces, normal text, links, selection, focus and native controls."));
 auto edit=new QLineEdit("Selected project name and visible native focus");layout->addWidget(edit);
 auto list=new QListWidget;list->addItems({"Current destination","A longer navigation label that remains readable","Background and text roles"});list->setCurrentRow(0);layout->addWidget(list);
 auto check=new QCheckBox("Native checkbox: text remains a status cue");check->setChecked(true);layout->addWidget(check);
 auto combo=new QComboBox;combo->addItems({"First option","Second option","Third option"});layout->addWidget(combo);
 auto link=new QLabel("<a href='https://example.invalid'>Native application link (preview only)</a>");link->setOpenExternalLinks(false);layout->addWidget(link);
 auto progress=new QProgressBar;progress->setValue(45);layout->addWidget(progress);
 layout->addWidget(new QPushButton("Native button"));auto disabled=new QPushButton("Unavailable control");disabled->setEnabled(false);layout->addWidget(disabled);
 window.show();edit->setFocus();edit->selectAll();
 QTimer::singleShot(250,[&]{
  if(!window.grab().save(output+"-before.png"))app.exit(3);
  auto config=KSharedConfig::openConfig(scheme,KConfig::SimpleConfig);
  app.setProperty("KDE_COLOR_SCHEME_PATH",scheme);
  app.setPalette(KColorScheme::createApplicationPalette(config));
  window.update();edit->setFocus();edit->selectAll();
  QTimer::singleShot(250,[&]{
   if(!window.grab().save(output+"-after.png")){app.exit(3);return;}
   auto palette=app.palette();QJsonObject colors;
   for(auto pair: {std::pair{"Window",QPalette::Window}, {"WindowText",QPalette::WindowText},{"Base",QPalette::Base},{"Text",QPalette::Text},{"Button",QPalette::Button},{"ButtonText",QPalette::ButtonText},{"Highlight",QPalette::Highlight},{"HighlightedText",QPalette::HighlightedText},{"Link",QPalette::Link}}) colors[pair.first]=palette.color(QPalette::Active,pair.second).name();
   QJsonObject result{{"qt",qVersion()},{"style",app.style()->metaObject()->className()},{"font",app.font().family()},{"scheme",scheme},{"colors",colors},{"focus",edit->hasFocus()},{"selectedText",edit->selectedText()}};
   QFile file(output+".json");if(!file.open(QIODevice::WriteOnly)){app.exit(4);return;}file.write(QJsonDocument(result).toJson());file.close();app.quit();
  });
 });
 return app.exec();
}
