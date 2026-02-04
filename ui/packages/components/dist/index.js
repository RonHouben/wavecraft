import { jsxs, jsx, Fragment } from "react/jsx-runtime";
import { useState, useRef, useEffect, useCallback } from "react";
import { useConnectionStatus, getMeterFrame, linearToDb, useParameter, logger, useLatencyMonitor, useRequestResize } from "@wavecraft/core";
const METER_UPDATE_HZ = 30;
const METER_FLOOR_DB = -60;
const METER_RANGE_DB = 60;
const CLIP_THRESHOLD = 1;
const CLIP_HOLD_MS = 2e3;
function Meter() {
  const { connected } = useConnectionStatus();
  const [frame, setFrame] = useState(null);
  const [clippedL, setClippedL] = useState(false);
  const [clippedR, setClippedR] = useState(false);
  const clipLTimeoutRef = useRef(null);
  const clipRTimeoutRef = useRef(null);
  useEffect(() => {
    if (!connected) {
      return;
    }
    const interval = setInterval(async () => {
      const newFrame = await getMeterFrame();
      setFrame(newFrame);
      if (newFrame) {
        if (newFrame.peak_l > CLIP_THRESHOLD) {
          setClippedL(true);
          if (clipLTimeoutRef.current !== null) {
            clearTimeout(clipLTimeoutRef.current);
          }
          clipLTimeoutRef.current = globalThis.setTimeout(() => {
            setClippedL(false);
            clipLTimeoutRef.current = null;
          }, CLIP_HOLD_MS);
        }
        if (newFrame.peak_r > CLIP_THRESHOLD) {
          setClippedR(true);
          if (clipRTimeoutRef.current !== null) {
            clearTimeout(clipRTimeoutRef.current);
          }
          clipRTimeoutRef.current = globalThis.setTimeout(() => {
            setClippedR(false);
            clipRTimeoutRef.current = null;
          }, CLIP_HOLD_MS);
        }
      }
    }, 1e3 / METER_UPDATE_HZ);
    return () => {
      clearInterval(interval);
      if (clipLTimeoutRef.current !== null) {
        clearTimeout(clipLTimeoutRef.current);
      }
      if (clipRTimeoutRef.current !== null) {
        clearTimeout(clipRTimeoutRef.current);
      }
    };
  }, [connected]);
  const peakLDb = frame ? linearToDb(frame.peak_l, METER_FLOOR_DB) : METER_FLOOR_DB;
  const peakRDb = frame ? linearToDb(frame.peak_r, METER_FLOOR_DB) : METER_FLOOR_DB;
  const rmsLDb = frame ? linearToDb(frame.rms_l, METER_FLOOR_DB) : METER_FLOOR_DB;
  const rmsRDb = frame ? linearToDb(frame.rms_r, METER_FLOOR_DB) : METER_FLOOR_DB;
  const peakLPercent = (peakLDb - METER_FLOOR_DB) / METER_RANGE_DB * 100;
  const peakRPercent = (peakRDb - METER_FLOOR_DB) / METER_RANGE_DB * 100;
  const rmsLPercent = (rmsLDb - METER_FLOOR_DB) / METER_RANGE_DB * 100;
  const rmsRPercent = (rmsRDb - METER_FLOOR_DB) / METER_RANGE_DB * 100;
  const handleResetClip = () => {
    setClippedL(false);
    setClippedR(false);
    if (clipLTimeoutRef.current !== null) {
      clearTimeout(clipLTimeoutRef.current);
      clipLTimeoutRef.current = null;
    }
    if (clipRTimeoutRef.current !== null) {
      clearTimeout(clipRTimeoutRef.current);
      clipRTimeoutRef.current = null;
    }
  };
  if (!connected) {
    return /* @__PURE__ */ jsxs(
      "div",
      {
        "data-testid": "meter",
        className: "flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4 font-sans",
        children: [
          /* @__PURE__ */ jsx("div", { className: "flex items-center justify-between gap-2", children: /* @__PURE__ */ jsx("div", { className: "text-xs font-semibold uppercase tracking-wide text-gray-500", children: "Levels" }) }),
          /* @__PURE__ */ jsx("div", { className: "flex items-center justify-center py-8 text-sm text-gray-400", children: "⏳ Connecting..." })
        ]
      }
    );
  }
  return /* @__PURE__ */ jsxs(
    "div",
    {
      "data-testid": "meter",
      className: "flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4 font-sans",
      children: [
        /* @__PURE__ */ jsxs("div", { className: "flex items-center justify-between gap-2", children: [
          /* @__PURE__ */ jsx("div", { className: "text-xs font-semibold uppercase tracking-wide text-gray-500", children: "Levels" }),
          (clippedL || clippedR) && /* @__PURE__ */ jsx(
            "button",
            {
              "data-testid": "meter-clip-button",
              className: "animate-clip-pulse cursor-pointer select-none rounded border-none bg-meter-clip px-2 py-0.5 text-[10px] font-bold text-white hover:bg-meter-clip-dark active:scale-95",
              onClick: handleResetClip,
              title: "Click to reset",
              type: "button",
              children: "CLIP"
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("div", { "data-testid": "meter-L", className: "flex items-center gap-2 rounded bg-plugin-dark p-2", children: [
          /* @__PURE__ */ jsx("div", { className: "w-4 text-center text-[11px] font-semibold text-gray-300", children: "L" }),
          /* @__PURE__ */ jsx("div", { className: "relative h-6 flex-1", children: /* @__PURE__ */ jsxs(
            "div",
            {
              className: `relative h-full w-full overflow-hidden rounded bg-[#333] transition-shadow duration-100 ${clippedL ? "shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]" : ""}`,
              children: [
                /* @__PURE__ */ jsx(
                  "div",
                  {
                    "data-testid": "meter-L-rms",
                    className: "absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light transition-[width] duration-100",
                    style: { width: `${Math.max(0, Math.min(100, rmsLPercent))}%` }
                  }
                ),
                /* @__PURE__ */ jsx(
                  "div",
                  {
                    "data-testid": "meter-L-peak",
                    className: "duration-50 absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 transition-[width]",
                    style: { width: `${Math.max(0, Math.min(100, peakLPercent))}%` }
                  }
                )
              ]
            }
          ) }),
          /* @__PURE__ */ jsxs(
            "div",
            {
              "data-testid": "meter-L-db",
              className: `w-[60px] text-right font-mono text-[11px] text-gray-300 transition-colors duration-100 ${clippedL ? "font-semibold text-meter-clip" : ""}`,
              children: [
                peakLDb.toFixed(1),
                " dB"
              ]
            }
          )
        ] }),
        /* @__PURE__ */ jsxs("div", { "data-testid": "meter-R", className: "flex items-center gap-2 rounded bg-plugin-dark p-2", children: [
          /* @__PURE__ */ jsx("div", { className: "w-4 text-center text-[11px] font-semibold text-gray-300", children: "R" }),
          /* @__PURE__ */ jsx("div", { className: "relative h-6 flex-1", children: /* @__PURE__ */ jsxs(
            "div",
            {
              className: `relative h-full w-full overflow-hidden rounded bg-[#333] transition-shadow duration-100 ${clippedR ? "shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]" : ""}`,
              children: [
                /* @__PURE__ */ jsx(
                  "div",
                  {
                    "data-testid": "meter-R-rms",
                    className: "absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light transition-[width] duration-100",
                    style: { width: `${Math.max(0, Math.min(100, rmsRPercent))}%` }
                  }
                ),
                /* @__PURE__ */ jsx(
                  "div",
                  {
                    "data-testid": "meter-R-peak",
                    className: "duration-50 absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 transition-[width]",
                    style: { width: `${Math.max(0, Math.min(100, peakRPercent))}%` }
                  }
                )
              ]
            }
          ) }),
          /* @__PURE__ */ jsxs(
            "div",
            {
              "data-testid": "meter-R-db",
              className: `w-[60px] text-right font-mono text-[11px] text-gray-300 transition-colors duration-100 ${clippedR ? "font-semibold text-meter-clip" : ""}`,
              children: [
                peakRDb.toFixed(1),
                " dB"
              ]
            }
          )
        ] })
      ]
    }
  );
}
function ParameterSlider({ id }) {
  const { param, setValue, isLoading, error } = useParameter(id);
  if (isLoading) {
    return /* @__PURE__ */ jsxs("div", { className: "mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4 italic text-gray-500", children: [
      "Loading ",
      id,
      "..."
    ] });
  }
  if (error || !param) {
    return /* @__PURE__ */ jsxs("div", { className: "mb-4 rounded-lg border border-red-400 bg-plugin-surface p-4 text-red-400", children: [
      "Error: ",
      (error == null ? void 0 : error.message) || "Parameter not found"
    ] });
  }
  const handleChange = (e) => {
    const value = Number.parseFloat(e.target.value);
    setValue(value).catch((err) => {
      logger.error("Failed to set parameter", { error: err, parameterId: id });
    });
  };
  const unitSuffix = param.unit === "%" ? param.unit : ` ${param.unit}`;
  const displayValue = param.unit ? `${(param.value * 100).toFixed(1)}${unitSuffix}` : param.value.toFixed(3);
  return /* @__PURE__ */ jsxs(
    "div",
    {
      "data-testid": `param-${id}`,
      className: "mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4",
      children: [
        /* @__PURE__ */ jsxs("div", { className: "mb-2 flex items-center justify-between", children: [
          /* @__PURE__ */ jsx(
            "label",
            {
              "data-testid": `param-${id}-label`,
              htmlFor: `slider-${id}`,
              className: "font-semibold text-gray-200",
              children: param.name
            }
          ),
          /* @__PURE__ */ jsx("span", { "data-testid": `param-${id}-value`, className: "font-mono text-sm text-accent", children: displayValue })
        ] }),
        /* @__PURE__ */ jsx(
          "input",
          {
            "data-testid": `param-${id}-slider`,
            id: `slider-${id}`,
            type: "range",
            min: "0",
            max: "1",
            step: "0.001",
            value: param.value,
            onChange: handleChange,
            className: "slider-thumb h-1.5 w-full appearance-none rounded-sm bg-plugin-border outline-none"
          }
        )
      ]
    }
  );
}
function ParameterGroup({ group }) {
  return /* @__PURE__ */ jsxs("div", { className: "space-y-2", children: [
    /* @__PURE__ */ jsx("h3", { className: "text-sm font-semibold uppercase tracking-wider text-gray-400", children: group.name }),
    /* @__PURE__ */ jsx("div", { className: "space-y-3", children: group.parameters.map((param) => /* @__PURE__ */ jsx(ParameterSlider, { id: param.id }, param.id)) })
  ] });
}
function ParameterToggle({ id }) {
  const { param, setValue, isLoading, error } = useParameter(id);
  if (isLoading) {
    return /* @__PURE__ */ jsxs("div", { className: "mb-4 flex items-center justify-between rounded-lg border border-plugin-border bg-plugin-surface p-4 italic text-gray-500", children: [
      "Loading ",
      id,
      "..."
    ] });
  }
  if (error || !param) {
    return /* @__PURE__ */ jsxs("div", { className: "mb-4 flex items-center justify-between rounded-lg border border-red-400 bg-plugin-surface p-4 text-red-400", children: [
      "Error: ",
      (error == null ? void 0 : error.message) || "Parameter not found"
    ] });
  }
  const isOn = param.value >= 0.5;
  const handleToggle = () => {
    const newValue = isOn ? 0 : 1;
    setValue(newValue).catch((err) => {
      logger.error("Failed to set parameter", { error: err, parameterId: id });
    });
  };
  return /* @__PURE__ */ jsxs("div", { className: "mb-4 flex items-center justify-between rounded-lg border border-plugin-border bg-plugin-surface p-4", children: [
    /* @__PURE__ */ jsx("label", { htmlFor: `toggle-${id}`, className: "font-semibold text-gray-200", children: param.name }),
    /* @__PURE__ */ jsx(
      "button",
      {
        id: `toggle-${id}`,
        className: `relative h-[26px] w-[50px] cursor-pointer rounded-full border-none outline-none transition-colors duration-200 ${isOn ? "bg-accent hover:bg-accent-light" : "bg-gray-600 hover:bg-gray-500"}`,
        onClick: handleToggle,
        "aria-pressed": isOn,
        children: /* @__PURE__ */ jsx(
          "span",
          {
            className: `absolute top-[3px] h-5 w-5 rounded-full bg-white transition-[left] duration-200 ${isOn ? "left-[27px]" : "left-[3px]"}`
          }
        )
      }
    )
  ] });
}
function VersionBadge() {
  return /* @__PURE__ */ jsxs("span", { "data-testid": "version-badge", className: "text-sm font-medium text-accent", children: [
    "v",
    __APP_VERSION__
  ] });
}
function ConnectionStatus() {
  const { connected, transport } = useConnectionStatus();
  if (transport === "native") {
    return /* @__PURE__ */ jsx(Fragment, {});
  }
  return /* @__PURE__ */ jsxs(
    "div",
    {
      "data-testid": "connection-status",
      className: `flex items-center gap-2 rounded px-3 py-1.5 text-sm ${connected ? "bg-green-900/30 text-green-400" : "bg-yellow-900/30 text-yellow-400"}`,
      children: [
        /* @__PURE__ */ jsx("div", { className: `h-2 w-2 rounded-full ${connected ? "bg-green-400" : "bg-yellow-400"}` }),
        /* @__PURE__ */ jsx("span", { children: connected ? "Connected" : "Connecting..." }),
        transport === "websocket" && /* @__PURE__ */ jsx("span", { className: "text-xs opacity-70", children: "(WebSocket)" })
      ]
    }
  );
}
function LatencyMonitor() {
  const { latency, avg, max, count } = useLatencyMonitor(1e3);
  const getStatusColor = () => {
    if (avg < 5) return "text-green-400";
    if (avg < 10) return "text-yellow-400";
    return "text-red-400";
  };
  const getStatusText = () => {
    if (avg < 5) return "✓ Excellent";
    if (avg < 10) return "⚠ Fair";
    return "✗ Poor";
  };
  return /* @__PURE__ */ jsxs("div", { className: "mb-4 rounded-lg border border-plugin-border bg-plugin-surface p-4", children: [
    /* @__PURE__ */ jsx("h3", { className: "m-0 mb-3 text-base font-semibold text-gray-200", children: "IPC Latency" }),
    /* @__PURE__ */ jsxs("div", { className: "grid grid-cols-2 gap-2", children: [
      /* @__PURE__ */ jsxs("div", { className: "flex justify-between rounded bg-plugin-dark p-2", children: [
        /* @__PURE__ */ jsx("span", { className: "text-sm text-gray-500", children: "Current:" }),
        /* @__PURE__ */ jsx("span", { className: "font-mono text-sm font-semibold text-accent", children: latency === null ? "—" : `${latency.toFixed(2)} ms` })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: "flex justify-between rounded bg-plugin-dark p-2", children: [
        /* @__PURE__ */ jsx("span", { className: "text-sm text-gray-500", children: "Average:" }),
        /* @__PURE__ */ jsx("span", { className: "font-mono text-sm font-semibold text-accent", children: avg > 0 ? `${avg.toFixed(2)} ms` : "—" })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: "flex justify-between rounded bg-plugin-dark p-2", children: [
        /* @__PURE__ */ jsx("span", { className: "text-sm text-gray-500", children: "Max:" }),
        /* @__PURE__ */ jsx("span", { className: "font-mono text-sm font-semibold text-accent", children: max > 0 ? `${max.toFixed(2)} ms` : "—" })
      ] }),
      /* @__PURE__ */ jsxs("div", { className: "flex justify-between rounded bg-plugin-dark p-2", children: [
        /* @__PURE__ */ jsx("span", { className: "text-sm text-gray-500", children: "Samples:" }),
        /* @__PURE__ */ jsx("span", { className: "font-mono text-sm font-semibold text-accent", children: count })
      ] })
    ] }),
    /* @__PURE__ */ jsx("div", { className: "mt-3 text-center text-sm font-semibold", children: avg > 0 && /* @__PURE__ */ jsx("span", { className: getStatusColor(), children: getStatusText() }) })
  ] });
}
function ResizeHandle() {
  const requestResize = useRequestResize();
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, width: 0, height: 0 });
  const handleMouseDown = useCallback(
    (e) => {
      e.preventDefault();
      setIsDragging(true);
      dragStartRef.current = {
        x: e.clientX,
        y: e.clientY,
        width: window.innerWidth,
        height: window.innerHeight
      };
      const handleMouseMove = (moveEvent) => {
        const deltaX = moveEvent.clientX - dragStartRef.current.x;
        const deltaY = moveEvent.clientY - dragStartRef.current.y;
        const newWidth = Math.max(400, dragStartRef.current.width + deltaX);
        const newHeight = Math.max(300, dragStartRef.current.height + deltaY);
        requestResize(newWidth, newHeight).catch((err) => {
          logger.error("Resize request failed", { error: err, width: newWidth, height: newHeight });
        });
      };
      const handleMouseUp = () => {
        setIsDragging(false);
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [requestResize]
  );
  return /* @__PURE__ */ jsx(
    "button",
    {
      "data-testid": "resize-handle",
      className: `group fixed bottom-1 right-5 z-[9999] flex h-9 w-9 cursor-nwse-resize select-none items-center justify-center rounded border-none bg-transparent p-0 transition-colors duration-150 ${isDragging ? "bg-accent/20" : "hover:bg-white/10"}`,
      onMouseDown: handleMouseDown,
      "aria-label": "Resize window",
      type: "button",
      children: /* @__PURE__ */ jsxs(
        "svg",
        {
          width: "20",
          height: "20",
          viewBox: "0 0 16 16",
          fill: "none",
          xmlns: "http://www.w3.org/2000/svg",
          className: `transition-colors duration-150 ${isDragging ? "text-accent-light" : "text-white/50 group-hover:text-accent"}`,
          children: [
            /* @__PURE__ */ jsx(
              "line",
              {
                x1: "14",
                y1: "2",
                x2: "2",
                y2: "14",
                stroke: "currentColor",
                strokeWidth: "1.5",
                strokeLinecap: "round"
              }
            ),
            /* @__PURE__ */ jsx(
              "line",
              {
                x1: "14",
                y1: "6",
                x2: "6",
                y2: "14",
                stroke: "currentColor",
                strokeWidth: "1.5",
                strokeLinecap: "round"
              }
            ),
            /* @__PURE__ */ jsx(
              "line",
              {
                x1: "14",
                y1: "10",
                x2: "10",
                y2: "14",
                stroke: "currentColor",
                strokeWidth: "1.5",
                strokeLinecap: "round"
              }
            )
          ]
        }
      )
    }
  );
}
const PRESET_SIZES = [
  { name: "Small", width: 600, height: 400 },
  { name: "Medium", width: 800, height: 600 },
  { name: "Large", width: 1024, height: 768 },
  { name: "Extra Large", width: 1280, height: 960 }
];
function ResizeControls() {
  const requestResize = useRequestResize();
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const handleResize = async (width, height) => {
    setIsLoading(true);
    setStatus(`Requesting ${width}x${height}...`);
    try {
      const accepted = await requestResize(width, height);
      if (accepted) {
        setStatus(`✓ Resized to ${width}x${height}`);
      } else {
        setStatus(`✗ Host rejected ${width}x${height}`);
      }
    } catch (error) {
      setStatus(`✗ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };
  return /* @__PURE__ */ jsxs("div", { className: "rounded-lg bg-black/5 p-5", children: [
    /* @__PURE__ */ jsx("h3", { className: "m-0 mb-4 text-sm font-semibold uppercase tracking-wide text-black/70", children: "Window Size" }),
    /* @__PURE__ */ jsx("div", { className: "mb-4 grid grid-cols-2 gap-2.5", children: PRESET_SIZES.map((preset) => /* @__PURE__ */ jsxs(
      "button",
      {
        onClick: () => handleResize(preset.width, preset.height),
        disabled: isLoading,
        className: "flex cursor-pointer flex-col items-center justify-center rounded-md border border-gray-300 bg-white p-3 font-medium text-gray-800 transition-all duration-200 hover:-translate-y-px hover:border-blue-500 hover:bg-gray-100 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-50",
        children: [
          preset.name,
          /* @__PURE__ */ jsxs("span", { className: "mt-1 text-[11px] text-gray-500", children: [
            preset.width,
            " × ",
            preset.height
          ] })
        ]
      },
      preset.name
    )) }),
    status && /* @__PURE__ */ jsx(
      "div",
      {
        className: `rounded px-3 py-2 text-center text-sm ${(() => {
          if (status.startsWith("✓")) return "bg-green-500/10 text-green-600";
          if (status.startsWith("✗")) return "bg-red-500/10 text-red-500";
          return "bg-black/5 text-gray-500";
        })()}`,
        children: status
      }
    )
  ] });
}
export {
  ConnectionStatus,
  LatencyMonitor,
  Meter,
  ParameterGroup,
  ParameterSlider,
  ParameterToggle,
  ResizeControls,
  ResizeHandle,
  VersionBadge
};
//# sourceMappingURL=index.js.map
