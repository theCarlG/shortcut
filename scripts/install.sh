#!/bin/bash -eu
[ "$UID" -eq 0 ] || exec sudo "$0" "$@"

echo "Get latest release"
URL=$(curl -LH "Accept: application/vnd.github+json" https://api.github.com/repos/theCarlG/shortcut/releases/latest | jq -r '.assets[].browser_download_url')
RELEASE=$(basename $(echo $URL | sed 's|\.tar\.gz||g'))

echo "Download release $RELEASE"
curl -L $URL > /tmp/$RELEASE.tar.gz && cd /tmp && tar -xzf $RELEASE.tar.gz
cd $RELEASE

DEST=/home/deck/.local/bin
test ! -d $DEST && mkdir -p $DEST && chown -R deck: $DEST

echo "Stopping shortcut_daemon service (if exists)"
systemctl stop shortcut_daemon 2> /dev/null
echo "Disabling shortcut_daemon service (if exists)"
systemctl disable shortcut_daemon 2> /dev/null

echo "Installing files"
for bin in shortcut-gui shortcut-daemon; do
    if [ -f $DEST/$bin ]; then
        rm -f $DEST/$bin
    fi
    if [ -f $bin ]; then
        cp -v $bin $DEST/$bin
        chown deck: $DEST/$bin
    fi
done


echo "Installing shortcut_daemon service"
cat > /etc/systemd/system/shortcut_daemon.service <<- EOM
[Unit]
Description=Steam Deck Shortcut Deamon
[Service]
Type=simple
User=root
Restart=always
ExecStart=/home/deck/.local/bin/shortcut-daemon
WorkingDirectory=/home/deck/
[Install]
WantedBy=multi-user.target
EOM

echo "Enabling shortcut_daemon service"
systemctl daemon-reload
systemctl enable shortcut_daemon
systemctl start shortcut_daemon

echo "Installing Shortcut desktop entry"
cat > /home/deck/.local/share/applications/Shortcut.desktop <<- EOM
[Desktop Entry]
Name=Shortcut
Exec=/home/deck/.local/bin/shortcut-gui
Type=Application
Categories=Settings;Utility;
EOM

exit 0
