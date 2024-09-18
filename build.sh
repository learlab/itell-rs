cargo run --bin fetch_volume 9
rm -rf ../itell-strapi-demo/apps/research-methods-in-psychology/content/textbook/*.md
cp -r output/*md ../itell-strapi-demo/apps/research-methods-in-psychology/content/textbook/
