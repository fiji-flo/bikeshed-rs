document.body.addEventListener("click", (event) => {
  const queryAll = (sel) => [].slice.call(document.querySelectorAll(sel));

  // Find the dfn element or panel, if any, that was clicked on.
  let el = event.target;
  let target;
  let hitALink = false;

  while (el.parentElement) {
    if (el.tagName === "A") {
      // Clicking on a link in a <dfn> shouldn't summon the panel.
      hitALink = true;
    }

    if (el.classList.contains("dfn-paneled")) {
      target = "dfn";
      break;
    }

    if (el.classList.contains("dfn-panel")) {
      target = "dfn-panel";
      break;
    }

    el = el.parentElement;
  }

  if (target !== "dfn-panel") {
    // Turn off any currently "on" or "activated" panels.
    queryAll(".dfn-panel.on, .dfn-panel.activated").forEach((el) => {
      el.classList.remove("on");
      el.classList.remove("activated");
    });
  }

  if (target === "dfn" && !hitALink) {
    // Open the panel.
    const dfnPanel = document.querySelector(".dfn-panel[data-for='" + el.id + "']");

    if (dfnPanel) {
      dfnPanel.classList.add("on");
      const rect = el.getBoundingClientRect();
      dfnPanel.style.left = window.scrollX + rect.right + 5 + "px";
      dfnPanel.style.top = window.scrollY + rect.top + "px";
      const panelRect = dfnPanel.getBoundingClientRect();
      const panelWidth = panelRect.right - panelRect.left;

      if (panelRect.right > document.body.scrollWidth && (rect.left - (panelWidth + 5)) > 0) {
        // Reposition, because the panel is overflowing.
        dfnPanel.style.left = window.scrollX + rect.left - (panelWidth + 5) + "px";
      }
    } else {
      console.log("Couldn't find .dfn-panel[data-for='" + el.id + "']");
    }
  } else if (target === "dfn-panel") {
    // Switch it to "activated" state, which pins it.
    el.classList.add("activated");
    el.style.left = null;
    el.style.top = null;
  }
});
