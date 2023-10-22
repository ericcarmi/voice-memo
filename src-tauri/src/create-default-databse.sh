# start sqlite3
# sqlite3 default.db

CREATE TABLE prefs(id INTEGER PRIMARY KEY, darkmode bool, accent varchar(255), alwaysOnTop bool);
INSERT into prefs (id, darkmode, accent, alwaysOnTop) VALUES (0, true, 'orange', false);
# SELECT * from prefs;
# verify?

# cp .\default.db .\assets\cv_voicecontrol_database.sqlite # only need this one?
# cp .\default.db .\target\debug\assets\cv_voicecontrol_database.sqlite
