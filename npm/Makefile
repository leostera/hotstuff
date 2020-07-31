GITHUB_REGISTRY=https://npm.pkg.github.com

test:
	rm -rf node_modules
	yarn

publish: README.md publish.npm publish.github

README.md: ../README.md
	cp ../README.md .

publish.npm:
	npm publish --access public

publish.github:
	mv package.json npm-package.json
	cat npm-package.json \
		| sed '/name/s/@abstractmachines/@abstractmachineslab/g' \
		> package.json
	npm publish --registry $(GITHUB_REGISTRY)
	rm package.json
	mv npm-package.json package.json
