import 'dart:convert';
import 'dart:io';
import 'package:translator/translator.dart';

const int concurrencyLimit = 5;

/// Preserve original words that contain 2+ uppercase letters (e.g. RPC, WALA)
String preserveUppercaseWords(String original, String translated) {
  final acronymWords = original.split(RegExp(r'\s+')).where((word) {
    final caps = word.runes.where((r) => r >= 65 && r <= 90).length;
    return caps >= 2;
  });

  for (final word in acronymWords) {
    final regex = RegExp(
      r'\b' + RegExp.escape(word.toLowerCase()) + r'\b',
      caseSensitive: false,
    );
    translated = translated.replaceAllMapped(regex, (_) => word);
  }

  return translated;
}

Future<void> main() async {
  final file = File('core/resources/i18n/i18n.json');
  if (!await file.exists()) {
    stderr.writeln('❌ Error: Cannot find file at ${file.path}');
    exit(1);
  }

  final raw = await file.readAsString();
  final jsonData = jsonDecode(raw);

  final languagesList = (jsonData['languages'] as Map).keys.toList();
  final translationsSection = jsonData['translations'] as Map;

  final translations = <String, Map<String, dynamic>>{};
  for (final entry in translationsSection.entries) {
    translations[entry.key] = Map<String, dynamic>.from(entry.value);
  }

  final source = translations['en'];
  if (source == null) {
    stderr.writeln('❌ Error: No "en" base translation found.');
    exit(1);
  }

  final sourceKeys = source.keys.toSet();
  final translator = GoogleTranslator();
  var totalAdded = 0;

  for (final lang in languagesList) {
    if (lang == 'en') continue;

    final map = translations.putIfAbsent(lang, () {
      print('➕ Creating new translation block for "$lang"');
      return {};
    });

    final missing = sourceKeys.difference(map.keys.toSet());
    if (missing.isEmpty) {
      print('✅ "$lang" already complete — no keys missing.');
      continue;
    }

    print('\n🌐 Translating ${missing.length} keys into "$lang"...');

    final missingList = missing.toList();
    for (var i = 0; i < missingList.length; i += concurrencyLimit) {
      final batch = missingList.skip(i).take(concurrencyLimit);

      final futures = batch.map((key) async {
        final text = source[key];
        if (text == null || text.trim().isEmpty) return;

        try {
          final result = await translator.translate(text, from: 'en', to: lang);
          final adjusted = preserveUppercaseWords(text, result.text);
          map[key] = adjusted;
          totalAdded++;
          print('  ➕ $key → $adjusted');
        } catch (e) {
          stderr.writeln('  ❌ Failed to translate "$key" to $lang: $e');
        }
      });

      await Future.wait(futures);
    }
  }

  final dir = file.parent;
  if (!await dir.exists()) {
    await dir.create(recursive: true);
  }

  final backupPath = '${file.path}.bak';
  await File(backupPath).writeAsString(raw);
  print('\n🛡️  Backup saved to: $backupPath');

  jsonData['translations'] = translations;
  await file.writeAsString(JsonEncoder.withIndent('  ').convert(jsonData));
  print('✅ All translations saved to: ${file.path}');
  print('📝 $totalAdded new translations added.');
}
