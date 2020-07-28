<script>
  let assets =
  [
    ...document.querySelectorAll('[src]'),
    ...document.querySelectorAll('[href]'),
  ].reduce( (acc, $el) => {
    const attr = $el.src ? "src" : "href";
    acc[$el[attr]] = $el;
    return acc
  }, {
    [window.location.pathname]: Symbol.for('reload')
  });

  console.log(assets);

  let wait_for_changes = () =>  {
    fetch(`/___hotstuff___/reload`)
      .then(res => res.json())
      .then(({changes}) => {
        changes.forEach( path => {
          const name = `http://${window.location.host}${path}`;
          console.log("Reloading file", name);
          let $el = assets[name] || false
          if (assets[path] === Symbol.for('reload')) {
            window.location.reload()
          } else {
            const attr = $el.src ? "src" : "href";
            const last_value = $el[attr];
            console.log(attr, last_value);
            $el[attr] = "";
            $el[attr] = last_value;
          }
        });
        wait_for_changes();
      });
  };

  wait_for_changes();
</script>
