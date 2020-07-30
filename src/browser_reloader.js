<script>
console.log(`
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
██████░██ █▀▄▄▀█▄░▄███░▄▄█▄░▄█░██░█░▄▄█░▄▄██████
██████░▄▄░█░██░██░████▄▄▀██░██░██░█░▄██░▄███████
██████▄██▄██▄▄███▄████▄▄▄██▄███▄▄▄█▄███▄████████
▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
You're running the no-nonsense HotStuff server.
`);

let assets = [
  ...document.querySelectorAll("[src]"),
  ...document.querySelectorAll("[href]"),
].reduce(
  (acc, $el) => {
    const attr = $el.src ? "src" : "href";
    acc[$el[attr]] = $el;
    return acc;
  },
  {
    [window.location.pathname == "/"
      ? "/index.html"
      : window.location.pathname]: Symbol.for("reload"),
  }
);

console.log(`Listening on changes...`);

let wait_for_changes = () => {
  fetch(`/___hotstuff___/reload`)
    .then((res) => res.json())
    .then(({ changes }) => {
      changes.forEach((path) => {
        const name = `http://${window.location.host}${path}`;
        let $el = assets[name] || false;
        if (assets[path] === Symbol.for("reload")) {
          console.log("Reloading page...");
          window.location.reload();
        } else {
          console.log("Reloading file: ", name);
          const attr = $el.src ? "src" : "href";
          const last_value = $el[attr];
          $el[attr] = "";
          $el[attr] = last_value;
        }
      });
      wait_for_changes();
    });
};

wait_for_changes();
</script>
