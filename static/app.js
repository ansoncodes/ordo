/* Ordo — front-end application.
   Vanilla JS, no build step. State lives in `S`; every mutation goes through the
   Rust API and then re-fetches /api/bootstrap so scores, counts and views stay
   consistent with the server's priority engine. */
(() => {
  'use strict';

  // ------------------------------------------------------------------
  // Icons (Lucide-style strokes)
  // ------------------------------------------------------------------
  const ICONS = {
    plus: '<path d="M12 5v14M5 12h14"/>',
    x: '<path d="M18 6L6 18M6 6l12 12"/>',
    check: '<path d="M20 6L9 17l-5-5"/>',
    search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
    star: '<path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>',
    moon: '<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>',
    zap: '<path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>',
    calendar: '<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
    grid: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/>',
    inbox: '<path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>',
    'check-circle': '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/>',
    'bar-chart': '<path d="M12 20V10M18 20V4M6 20v-4"/>',
    folder: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
    tag: '<path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><path d="M7 7h.01"/>',
    settings: '<path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6"/>',
    clock: '<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>',
    repeat: '<path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>',
    trash: '<path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>',
    play: '<path d="M5 3l14 9-14 9V3z"/>',
    pause: '<rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/>',
    square: '<rect x="4" y="4" width="16" height="16" rx="2"/>',
    'chevron-right': '<path d="M9 18l6-6-6-6"/>',
    'chevron-down': '<path d="M6 9l6 6 6-6"/>',
    flag: '<path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><path d="M4 22v-7"/>',
    copy: '<rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    alert: '<circle cx="12" cy="12" r="10"/><path d="M12 8v4M12 16h.01"/>',
    download: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>',
    upload: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>',
    target: '<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>',
    restore: '<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>',
    edit: '<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4z"/>',
    layers: '<path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/>',
    list: '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
    'arrow-right': '<path d="M5 12h14M12 5l7 7-7 7"/>',
    hash: '<path d="M4 9h16M4 15h16M10 3L8 21M16 3l-2 18"/>',
    sparkles: '<path d="M12 3l1.9 5.7 5.7 1.9-5.7 1.9L12 18.2l-1.9-5.7L4.4 10.6l5.7-1.9z"/>',
    keyboard: '<rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M7 16h10"/>',
    command: '<path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/>',
    info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>',
    flame: '<path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/>',
    coffee: '<path d="M18 8h1a4 4 0 0 1 0 8h-1"/><path d="M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z"/><path d="M6 1v3M10 1v3M14 1v3"/>',
    filter: '<path d="M22 3H2l8 9.46V19l4 2v-8.54L22 3z"/>',
    skip: '<path d="M5 4l10 8-10 8V4z"/><path d="M19 5v14"/>',
    help: '<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3M12 17h.01"/>',
  };
  const icon = (name, cls = '') => `<svg viewBox="0 0 24 24" class="${cls}" aria-hidden="true">${ICONS[name] || ''}</svg>`;

  // ------------------------------------------------------------------
  // State
  // ------------------------------------------------------------------
  const S = {
    data: null,
    analytics: null,
    view: { type: 'today', id: null },
    selectedId: null,
    selection: new Set(),
    expanded: new Set(),
    showDone: {},
    sort: localStorage.getItem('ordo.sort') || 'score',
    search: '',
    searchResults: null,
    paletteOpen: false,
    paletteQuery: '',
    paletteIndex: 0,
    focusAfter: null,
    tableView: {},
    animateNext: true,
  };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  const VIEWS = [
    { id: 'today', label: 'Today', icon: 'sun', key: '1' },
    { id: 'focus', label: 'Focus', icon: 'zap', key: '2' },
    { id: 'upcoming', label: 'Upcoming', icon: 'calendar', key: '3' },
    { id: 'matrix', label: 'Priority Matrix', icon: 'grid', key: '4' },
    { id: 'inbox', label: 'Inbox', icon: 'inbox', key: '5' },
    { id: 'completed', label: 'Completed', icon: 'check-circle', key: '6' },
    { id: 'analytics', label: 'Analytics', icon: 'bar-chart', key: '7' },
    { id: 'trash', label: 'Trash', icon: 'trash', key: '8' },
  ];

  const PRIORITIES = [
    { id: 'p1', label: 'Critical', short: 'P1' },
    { id: 'p2', label: 'High', short: 'P2' },
    { id: 'p3', label: 'Medium', short: 'P3' },
    { id: 'p4', label: 'Low', short: 'P4' },
  ];

  const PROJECT_COLORS = ['#6366f1', '#10b981', '#f59e0b', '#ef4444', '#0ea5e9', '#8b5cf6', '#ec4899', '#14b8a6'];

  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));
  const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

  // ------------------------------------------------------------------
  // API
  // ------------------------------------------------------------------
  async function api(method, url, body) {
    const res = await fetch(url, {
      method,
      headers: body !== undefined ? { 'Content-Type': 'application/json' } : {},
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      let message = `${res.status} ${res.statusText}`;
      try { const j = await res.json(); if (j.error) message = j.error; } catch (_) { /* ignore */ }
      throw new Error(message);
    }
    const type = res.headers.get('content-type') || '';
    return type.includes('json') ? res.json() : res.text();
  }

  async function run(fn) {
    try { return await fn(); } catch (e) { toast(e.message || 'Something went wrong', { type: 'error' }); }
  }

  async function refresh() {
    S.data = await api('GET', '/api/bootstrap');
    if (S.view.type === 'analytics') S.analytics = await api('GET', '/api/analytics');
    applyTheme();
    render();
  }

  // ------------------------------------------------------------------
  // Data helpers
  // ------------------------------------------------------------------
  const allTasks = () => (S.data ? S.data.tasks : []);
  const liveTasks = () => allTasks().filter((t) => !t.deleted_at);
  const openTasks = () => liveTasks().filter((t) => !t.completed_at);
  const doneTasks = () => liveTasks().filter((t) => t.completed_at);
  const trashedTasks = () => allTasks().filter((t) => t.deleted_at);
  const today = () => S.data.today;
  const taskById = (id) => allTasks().find((t) => t.id === id);
  const projectById = (id) => (S.data ? S.data.projects.find((p) => p.id === id) : null);
  const isPlannedToday = (t) => !!t.scheduled && t.scheduled <= today();
  const isTodayTask = (t) => (!!t.due_date && t.due_date <= today()) || isPlannedToday(t);
  const settings = () => S.data.settings;

  function pad(n) { return String(n).padStart(2, '0'); }
  function parseDate(iso) { const [y, m, d] = iso.split('-').map(Number); return new Date(y, m - 1, d); }
  function toISO(d) { return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`; }
  function addDays(iso, n) { const d = parseDate(iso); d.setDate(d.getDate() + n); return toISO(d); }
  function daysBetween(a, b) { return Math.round((parseDate(b) - parseDate(a)) / 86400000); }
  function fmtDay(iso, opts) { return parseDate(iso).toLocaleDateString(undefined, opts || { weekday: 'short', day: 'numeric', month: 'short' }); }
  function fmtLongDay(iso) { return parseDate(iso).toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' }); }
  function weekdayName(iso) { return parseDate(iso).toLocaleDateString(undefined, { weekday: 'long' }); }
  function fmtTime(t) {
    if (!t) return '';
    const [h, m] = t.split(':').map(Number);
    const d = new Date(); d.setHours(h, m, 0, 0);
    return d.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
  }
  function fmtDateTime(iso) { return new Date(iso).toLocaleString(undefined, { day: 'numeric', month: 'short', hour: 'numeric', minute: '2-digit' }); }
  function localDate(iso) { return toISO(new Date(iso)); }
  function fmtMinutes(m) {
    m = Math.round(m || 0);
    if (m < 60) return `${m}m`;
    const h = Math.floor(m / 60), r = m % 60;
    return r ? `${h}h ${r}m` : `${h}h`;
  }
  function relDay(iso) {
    const n = daysBetween(today(), iso);
    if (n === 0) return 'Today';
    if (n === 1) return 'Tomorrow';
    if (n === -1) return 'Yesterday';
    if (n < 0) return `${-n}d overdue`;
    if (n < 7) return weekdayName(iso);
    return fmtDay(iso);
  }
  function dueLabel(t) {
    if (!t.due_date) return '';
    let label = relDay(t.due_date);
    if (t.due_status === 'overdue') label = `Overdue ${-t.days_until_due}d`;
    if (t.due_time) label += ` · ${fmtTime(t.due_time)}`;
    return label;
  }
  function scoreBand(s) { return s >= 65 ? 'hot' : s >= 40 ? 'warm' : 'cool'; }
  function quadrantLabel(q) { return { q1: 'Do first', q2: 'Schedule', q3: 'Do quickly', q4: 'Someday' }[q] || ''; }
  function initials(name) { return (name || 'Y').trim().split(/\s+/).map((w) => w[0]).slice(0, 2).join('').toUpperCase(); }

  const SORTERS = {
    score: (a, b) => b.score - a.score || cmpDue(a, b) || a.title.localeCompare(b.title),
    priority: (a, b) => a.priority.localeCompare(b.priority) || b.score - a.score,
    due: (a, b) => cmpDue(a, b) || b.score - a.score,
    created: (a, b) => new Date(b.created_at) - new Date(a.created_at),
    title: (a, b) => a.title.localeCompare(b.title),
  };
  function cmpDue(a, b) {
    if (!a.due_date && !b.due_date) return 0;
    if (!a.due_date) return 1;
    if (!b.due_date) return -1;
    return a.due_date.localeCompare(b.due_date) || (a.due_time || '').localeCompare(b.due_time || '');
  }
  const sorted = (list, key = S.sort) => list.slice().sort(SORTERS[key] || SORTERS.score);

  // ------------------------------------------------------------------
  // Theme
  // ------------------------------------------------------------------
  const ACCENTS = [
    { id: 'indigo', label: 'Indigo', swatch: '#6366f1' },
    { id: 'blue', label: 'Blue', swatch: '#3b82f6' },
    { id: 'violet', label: 'Violet', swatch: '#8b5cf6' },
    { id: 'teal', label: 'Teal', swatch: '#14b8a6' },
    { id: 'emerald', label: 'Emerald', swatch: '#10b981' },
    { id: 'amber', label: 'Amber', swatch: '#f59e0b' },
    { id: 'orange', label: 'Orange', swatch: '#f97316' },
    { id: 'rose', label: 'Rose', swatch: '#f43f5e' },
    { id: 'neutral', label: 'Neutral', swatch: 'linear-gradient(135deg, #171717, #a3a3a3)' },
  ];
  const DARK_STYLES = [
    { id: 'soft', label: 'Soft dark', desc: 'Blue-tinted surfaces, gentle contrast', bg: '#0d1017', side: '#141925', card: '#1b2130' },
    { id: 'plain', label: 'Plain dark', desc: 'Pure black, hairline borders', bg: '#000000', side: '#0a0a0a', card: '#141414' },
  ];

  // --- colour helpers for custom accents ---
  const isHex = (s) => /^#[0-9a-f]{6}$/i.test(s || '');
  function hexToRgb(h) { const n = parseInt(h.slice(1), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; }
  function rgbToHex(rgb) { return `#${rgb.map((v) => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, '0')).join('')}`; }
  function mixRgb(a, b, t) { return a.map((v, i) => v + (b[i] - v) * t); }
  function luminance([r, g, b]) {
    const f = (c) => { c /= 255; return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; };
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
  }
  function rotateHue([r, g, b], deg) {
    r /= 255; g /= 255; b /= 255;
    const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
    let h = 0, s = 0;
    if (max !== min) {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      if (max === r) h = (g - b) / d + (g < b ? 6 : 0); else if (max === g) h = (b - r) / d + 2; else h = (r - g) / d + 4;
      h /= 6;
    }
    if (s === 0) return [l * 255, l * 255, l * 255];
    h = (h + deg / 360 + 1) % 1;
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s, p = 2 * l - q;
    const ch = (t) => { if (t < 0) t += 1; if (t > 1) t -= 1; if (t < 1 / 6) return p + (q - p) * 6 * t; if (t < 1 / 2) return q; if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6; return p; };
    return [ch(h + 1 / 3) * 255, ch(h) * 255, ch(h - 1 / 3) * 255];
  }
  /** Sets a custom accent, nudging it toward readability for the active theme. */
  function applyCustomAccent(hex, dark) {
    let rgb = hexToRgb(hex);
    const L = luminance(rgb);
    if (dark && L < 0.12) rgb = mixRgb(rgb, [255, 255, 255], 0.4);
    if (!dark && L > 0.5) rgb = mixRgb(rgb, [0, 0, 0], 0.35);
    const on = luminance(rgb) > 0.18 ? '#0b0b10' : '#ffffff';
    const st = document.documentElement.style;
    st.setProperty('--accent', rgbToHex(rgb));
    st.setProperty('--accent-2', rgbToHex(rotateHue(rgb, 35)));
    st.setProperty('--on-accent', on);
    st.setProperty('--on-brand', on);
  }
  const CUSTOM_WHEEL = 'conic-gradient(#f43f5e, #f59e0b, #84cc16, #14b8a6, #3b82f6, #8b5cf6, #f43f5e)';

  function appearance() {
    const s = (S.data && S.data.settings) || {};
    return {
      theme: s.theme || localStorage.getItem('ordo.theme') || 'system',
      dark_style: s.dark_style || localStorage.getItem('ordo.darkStyle') || 'soft',
      accent: s.accent || localStorage.getItem('ordo.accent') || 'indigo',
    };
  }
  function applyTheme() {
    const a = appearance();
    const dark = a.theme === 'dark' || (a.theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    const root = document.documentElement;
    root.classList.toggle('dark', dark);
    root.classList.toggle('plain', a.dark_style === 'plain');
    if (isHex(a.accent)) {
      root.dataset.accent = 'custom';
      applyCustomAccent(a.accent, dark);
    } else {
      root.dataset.accent = a.accent;
      ['--accent', '--accent-2', '--on-accent', '--on-brand'].forEach((p) => root.style.removeProperty(p));
    }
    const btn = $('#theme-btn');
    if (btn) btn.innerHTML = icon(dark ? 'sun' : 'moon');
  }
  /** Applies an appearance change instantly, remembers it locally, and persists it. */
  async function setAppearance(patch) {
    if (S.data) Object.assign(S.data.settings, patch);
    if (patch.theme) localStorage.setItem('ordo.theme', patch.theme);
    if (patch.dark_style) localStorage.setItem('ordo.darkStyle', patch.dark_style);
    if (patch.accent) localStorage.setItem('ordo.accent', patch.accent);
    applyTheme();
    await run(() => api('PATCH', '/api/settings', patch));
  }
  async function toggleTheme() {
    const dark = document.documentElement.classList.contains('dark');
    await setAppearance({ theme: dark ? 'light' : 'dark' });
  }
  async function toggleDarkStyle() {
    const next = appearance().dark_style === 'plain' ? 'soft' : 'plain';
    const patch = { dark_style: next };
    if (!document.documentElement.classList.contains('dark')) patch.theme = 'dark';
    await setAppearance(patch);
    toast(next === 'plain' ? 'Plain dark: pure black' : 'Soft dark: tinted surfaces');
  }
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);

  // ------------------------------------------------------------------
  // Routing
  // ------------------------------------------------------------------
  function viewToHash(v) {
    if (v.type === 'project') return `#/project/${v.id}`;
    if (v.type === 'tag') return `#/tag/${encodeURIComponent(v.id)}`;
    return `#/${v.type}`;
  }
  function hashToView() {
    const m = location.hash.match(/^#\/([a-z]+)(?:\/(.+))?$/);
    if (!m) return { type: 'today', id: null };
    const [, type, id] = m;
    if (type === 'project' || type === 'tag') return { type, id: type === 'tag' ? decodeURIComponent(id || '') : id };
    return VIEWS.some((v) => v.id === type) ? { type, id: null } : { type: 'today', id: null };
  }
  function go(view) {
    const hash = viewToHash(view);
    S.selection.clear();
    S.search = '';
    S.searchResults = null;
    const input = $('#search-input');
    if (input) input.value = '';
    if (location.hash === hash) { S.view = view; onViewChange(); } else { location.hash = hash; }
    $('#app').classList.remove('sidebar-open');
  }
  async function onViewChange() {
    if (S.view.type === 'analytics') S.analytics = await run(() => api('GET', '/api/analytics'));
    S.animateNext = true;
    render();
    $('#content').scrollTop = 0;
  }
  window.addEventListener('hashchange', () => { S.view = hashToView(); onViewChange(); });

  // ------------------------------------------------------------------
  // Rendering: shell
  // ------------------------------------------------------------------
  function render() {
    if (!S.data) return;
    renderSidebar();
    renderTopbar();
    renderContent();
    renderDetail();
    renderBulkBar();
    document.body.classList.toggle('selection-mode', S.selection.size > 0);
  }

  function counts() {
    const open = openTasks();
    const t = today();
    return {
      today: open.filter(isTodayTask).length,
      overdue: open.filter((x) => x.due_date && x.due_date < t).length,
      focus: open.length,
      upcoming: open.filter((x) => (x.due_date && x.due_date > t && daysBetween(t, x.due_date) <= 7) || (x.scheduled && x.scheduled > t && daysBetween(t, x.scheduled) <= 7)).length,
      matrix: open.filter((x) => x.quadrant === 'q1').length,
      inbox: open.filter((x) => !x.project_id).length,
      completed: doneTasks().filter((x) => localDate(x.completed_at) === t).length,
      analytics: null,
      trash: trashedTasks().length,
    };
  }

  function renderSidebar() {
    const c = counts();
    $('#nav-views').innerHTML = VIEWS.map((v) => {
      const n = c[v.id];
      const alert = v.id === 'today' && c.overdue > 0;
      return `<button class="nav-item ${S.view.type === v.id ? 'active' : ''}" data-action="go" data-view="${v.id}">
        ${icon(v.icon)}<span class="label">${v.label}</span>
        <span class="kbd-hint">${v.key}</span>
        ${n ? `<span class="count ${alert ? 'alert' : ''}" title="${alert ? `${c.overdue} overdue` : ''}">${n}</span>` : ''}
      </button>`;
    }).join('');

    const open = openTasks();
    $('#nav-projects').innerHTML = S.data.projects.map((p) => {
      const n = open.filter((t) => t.project_id === p.id).length;
      const active = S.view.type === 'project' && S.view.id === p.id;
      return `<button class="nav-item ${active ? 'active' : ''}" data-action="go" data-view="project" data-id="${p.id}">
        <i class="dot" style="background:${esc(p.color)}"></i><span class="label">${esc(p.name)}</span>
        <span class="edit-project" data-action="edit-project" data-id="${p.id}" title="Edit project">${icon('edit')}</span>
        ${n ? `<span class="count">${n}</span>` : ''}
      </button>`;
    }).join('') || '<div class="muted small" style="padding:6px 10px">No projects yet</div>';

    const tags = S.data.tags || [];
    $('#tags-section').hidden = tags.length === 0;
    $('#nav-tags').innerHTML = tags.slice(0, 24).map((t) => {
      const active = S.view.type === 'tag' && S.view.id === t.name;
      return `<button class="tag-pill ${active ? 'active' : ''}" data-action="go" data-view="tag" data-id="${esc(t.name)}">#${esc(t.name)}<span class="n">${t.count}</span></button>`;
    }).join('');

    const s = settings();
    $('#avatar').textContent = initials(s.user_name);
    $('#user-name').textContent = s.user_name || 'You';
    $('#workspace-name').textContent = s.workspace_name || 'My Workspace';
  }

  function viewMeta() {
    const v = S.view;
    const c = counts();
    if (S.search) return { title: `Search: “${S.search}”`, subtitle: `${(S.searchResults || []).length} matching tasks` };
    switch (v.type) {
      case 'today': return { title: 'Today', subtitle: `${fmtLongDay(today())} · ${c.today} task${c.today === 1 ? '' : 's'}${c.overdue ? ` · ${c.overdue} overdue` : ''}` };
      case 'focus': return { title: 'Focus', subtitle: `${c.focus} open tasks ranked by focus score` };
      case 'upcoming': return { title: 'Upcoming', subtitle: 'The next seven days, then everything later' };
      case 'matrix': return { title: 'Priority Matrix', subtitle: 'Importance from priority, urgency from deadlines and today’s plan. Drag to move.' };
      case 'inbox': return { title: 'Inbox', subtitle: `${c.inbox} task${c.inbox === 1 ? '' : 's'} without a project` };
      case 'completed': return { title: 'Completed', subtitle: `${doneTasks().length} done · ${c.completed} today` };
      case 'analytics': return { title: 'Analytics', subtitle: 'How your work is actually going' };
      case 'trash': return { title: 'Trash', subtitle: `${c.trash} deleted task${c.trash === 1 ? '' : 's'}` };
      case 'project': { const p = projectById(v.id); return { title: p ? p.name : 'Project', subtitle: p ? (p.description || `${openTasks().filter((t) => t.project_id === p.id).length} open tasks`) : 'This project no longer exists' }; }
      case 'tag': return { title: `#${v.id}`, subtitle: `${openTasks().filter((t) => t.tags.includes(v.id)).length} open tasks with this tag` };
      default: return { title: 'Ordo', subtitle: '' };
    }
  }

  function renderTopbar() {
    const m = viewMeta();
    $('#view-title').textContent = m.title;
    $('#view-subtitle').textContent = m.subtitle;
    const hideQuick = ['analytics', 'completed', 'trash'].includes(S.view.type) || !!S.search;
    $('#quick-add-wrap').hidden = hideQuick;
    const qa = $('#quick-add');
    const ph = {
      today: 'Add a task for today… e.g. "Reply to Maya 3pm !p2 ~15m"',
      project: `Add a task to ${esc(m.title)}… e.g. "Write spec by friday !p1 ~2h"`,
      tag: `Add a task tagged #${S.view.id}…`,
      upcoming: 'Add a task… e.g. "Dentist next tuesday 10am"',
      matrix: 'Add a task… e.g. "Renew licence in 2 weeks !p2"',
    }[S.view.type] || 'Add a task… try "Call Sam tomorrow 3pm !p1 #Personal ~30m"';
    qa.placeholder = ph.replace(/&quot;/g, '"');
    const aiBtn = $('.quick-add-ai');
    if (aiBtn) aiBtn.hidden = !settings().ai_enabled;
    applyTheme();
  }

  // ------------------------------------------------------------------
  // Rendering: task rows and lists
  // ------------------------------------------------------------------
  function taskRow(t, opts = {}) {
    const p = projectById(t.project_id);
    const done = !!t.completed_at;
    const cls = ['task', `prio-${t.priority}`, done ? 'done' : '', S.selectedId === t.id ? 'selected' : '', S.selection.has(t.id) ? 'checked' : ''].join(' ');
    const meta = [];
    if (p && !opts.hideProject) meta.push(`<span class="chip chip-project"><i class="dot" style="background:${esc(p.color)}"></i>${esc(p.name)}</span>`);
    if (t.due_date && !done) meta.push(`<span class="chip chip-due ${t.due_status}">${icon('calendar')}${esc(dueLabel(t))}</span>`);
    if (done) meta.push(`<span class="chip">${icon('check')}Done ${esc(fmtDateTime(t.completed_at))}</span>`);
    if (t.scheduled && !done && !opts.hidePlan) meta.push(`<span class="chip chip-plan">${icon('target')}${t.scheduled <= today() ? 'Planned today' : `Planned ${esc(relDay(t.scheduled))}`}</span>`);
    if (t.estimate_minutes) meta.push(`<span class="chip">${icon('clock')}${fmtMinutes(t.estimate_minutes)}</span>`);
    if (t.subtasks.length) meta.push(`<span class="chip">${icon('list')}${t.subtasks_done}/${t.subtasks.length}</span>`);
    if (t.recurrence) meta.push(`<span class="chip" title="${esc(t.recurrence_label)}">${icon('repeat')}${esc(t.recurrence_label)}</span>`);
    if (opts.showPriority) meta.push(`<span class="chip chip-prio" style="background:var(--${t.priority})">${t.priority.toUpperCase()}</span>`);
    t.tags.forEach((tag) => meta.push(`<span class="chip chip-tag" data-action="go" data-view="tag" data-id="${esc(tag)}">#${esc(tag)}</span>`));
    const reasons = opts.reasons !== false && S.expanded.has(t.id)
      ? `<div class="reasons">${t.reasons.map((r) => `<span class="reason">${esc(r)}</span>`).join('')}<span class="reason" style="background:var(--surface-3);color:var(--text-2)">${quadrantLabel(t.quadrant)}</span></div>`
      : '';
    const rank = opts.rank ? `<span class="rank-badge ${opts.rank <= 3 ? 'top' : ''}">${opts.rank}</span>` : '';
    const actions = t.deleted_at
      ? `<button class="icon-btn" data-action="restore" title="Restore">${icon('restore')}</button>
         <button class="icon-btn danger" data-action="purge" title="Delete forever">${icon('x')}</button>`
      : `<button class="icon-btn ${t.starred ? 'on' : ''}" data-action="star" title="Star">${icon('star')}</button>
         ${!done && !isPlannedToday(t) ? `<button class="icon-btn" data-action="plan-today" title="Plan for today">${icon('target')}</button>` : ''}
         ${!done ? `<button class="icon-btn" data-action="defer" title="Move to tomorrow">${icon('skip')}</button>` : ''}
         <button class="icon-btn" data-action="start-focus" title="Start focus session">${icon('play')}</button>
         <button class="icon-btn danger" data-action="trash" title="Move to trash">${icon('trash')}</button>`;
    const title = opts.highlight ? highlight(t.title, opts.highlight) : esc(t.title);
    return `<div class="${cls}" data-id="${t.id}" style="--i:${Math.min(opts.index || 0, 14)}" ${opts.draggable ? 'draggable="true"' : ''}>
      <button class="select-box" data-action="toggle-select" title="Select (Shift+click)">${icon('check')}</button>
      ${rank}
      <button class="check ${done ? 'done' : ''}" data-action="toggle-complete" title="${done ? 'Reopen' : 'Complete'}">${icon('check')}</button>
      <div class="task-body">
        <div class="task-title" title="Double-click to rename">${t.starred ? `<span class="star-on">${icon('star')}</span>` : ''}<span>${title}</span></div>
        ${meta.length ? `<div class="task-meta">${meta.join('')}</div>` : ''}
        ${reasons}
      </div>
      <div class="task-right">
        ${opts.score !== false && !done && !t.deleted_at ? `<button class="score ${scoreBand(t.score)}" data-action="toggle-reasons" title="Focus score — click to see why">${t.score}</button>` : ''}
        <div class="task-actions">${actions}</div>
      </div>
    </div>`;
  }

  function listCard(list, opts = {}) {
    if (!list.length) return emptyState(opts.empty || {});
    return `<div class="card"><div class="task-list">${list.map((t, i) => taskRow(t, { ...opts, index: i, rank: opts.ranked ? i + 1 : 0 })).join('')}</div></div>`;
  }

  /** Escapes text and wraps case-insensitive matches of `q` in <mark>. */
  function highlight(text, q) {
    const safe = esc(text);
    const needle = esc(q).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return needle ? safe.replace(new RegExp(needle, 'ig'), (m) => `<mark>${m}</mark>`) : safe;
  }

  function section(title, list, opts = {}) {
    if (!list.length && !opts.showEmpty) return '';
    return `<div class="section">
      <div class="section-header ${opts.danger ? 'danger' : ''}"><h3>${esc(title)}</h3><span class="n">${list.length}</span>${opts.extra || ''}</div>
      ${list.length ? listCard(list, opts) : `<div class="day-empty">${esc(opts.emptyText || 'Nothing here')}</div>`}
    </div>`;
  }

  function emptyState({ icon: ic = 'sparkles', title = 'All clear', text = '', cta = '' }) {
    return `<div class="card"><div class="empty"><div class="empty-icon">${icon(ic)}</div><h3>${esc(title)}</h3><p>${text}</p>${cta}</div></div>`;
  }

  function toolbar(count, extra = '') {
    return `<div class="row mb-16" style="justify-content:flex-end;gap:10px">
      ${extra}
      <span class="muted small">${count} task${count === 1 ? '' : 's'}</span>
      <label class="row small muted">Sort
        <select class="input input-sm" data-action="set-sort" style="width:auto">
          ${Object.entries({ score: 'Focus score', priority: 'Priority', due: 'Due date', created: 'Newest', title: 'Title' }).map(([k, v]) => `<option value="${k}" ${S.sort === k ? 'selected' : ''}>${v}</option>`).join('')}
        </select>
      </label>
    </div>`;
  }

  // ------------------------------------------------------------------
  // Rendering: views
  // ------------------------------------------------------------------
  function renderContent() {
    const el = $('#content');
    const top = el.scrollTop;
    let html = '';
    if (S.search) html = renderSearch();
    else {
      switch (S.view.type) {
        case 'today': html = renderToday(); break;
        case 'focus': html = renderFocus(); break;
        case 'upcoming': html = renderUpcoming(); break;
        case 'matrix': html = renderMatrix(); break;
        case 'inbox': html = renderInbox(); break;
        case 'completed': html = renderCompleted(); break;
        case 'analytics': html = renderAnalytics(); break;
        case 'trash': html = renderTrash(); break;
        case 'project': html = renderProject(); break;
        case 'tag': html = renderTag(); break;
      }
    }
    el.classList.toggle('animate', !!S.animateNext);
    S.animateNext = false;
    el.innerHTML = html;
    el.scrollTop = top;
  }

  function todayHero(list, doneToday) {
    const s = settings();
    const planned = list.reduce((a, t) => a + (t.estimate_minutes || s.default_estimate_minutes), 0);
    const cap = s.daily_capacity_minutes;
    const pct = Math.min(100, Math.round((planned / cap) * 100));
    const over = planned > cap;
    const warn = !over && pct >= 85;
    const total = list.length + doneToday.length;
    const donePct = total ? Math.round((doneToday.length / total) * 100) : 0;
    const hour = new Date().getHours();
    const greet = hour < 5 ? 'Still up' : hour < 12 ? 'Good morning' : hour < 17 ? 'Good afternoon' : hour < 21 ? 'Good evening' : 'Winding down';
    const first = (s.user_name || '').trim().split(/\s+/)[0];
    const name = first && first.toLowerCase() !== 'you' ? `, ${esc(first)}` : '';
    const ringLen = 2 * Math.PI * 27;
    const msg = !total ? 'Nothing on the list yet. Add a task, or let Ordo plan your day.' : doneToday.length === total ? 'Everything for today is done. Nicely played.' : `${doneToday.length} of ${total} done · ${list.length} to go`;
    const capMsg = over ? `over capacity by ${fmtMinutes(planned - cap)}. Defer something or shrink an estimate.` : cap - planned > 0 ? `${fmtMinutes(cap - planned)} of ${fmtMinutes(cap)} capacity free` : 'capacity fully planned';
    return `<div class="card hero">
      <div class="hero-ring" title="${doneToday.length} of ${total} done today"><svg viewBox="0 0 66 66"><circle class="track" cx="33" cy="33" r="27"/><circle class="fill" cx="33" cy="33" r="27" stroke-dasharray="${ringLen}" stroke-dashoffset="${ringLen * (1 - donePct / 100)}"/></svg><div class="num">${donePct}%</div></div>
      <div><h2>${greet}${name}</h2><p>${msg}</p>
        <div class="progress ${over ? 'over' : warn ? 'warn' : ''}"><i style="width:${pct}%"></i></div>
        <div class="cap">${fmtMinutes(planned)} planned · ${esc(capMsg)}</div></div>
      <div class="hero-actions"><button class="btn btn-primary" data-action="plan-day">${icon('sparkles')}Plan my day</button></div>
    </div>`;
  }

  function tipsCard() {
    if (localStorage.getItem('ordo.tipsDismissed')) return '';
    const tips = [
      ['edit', 'Type naturally', 'Try “Call Sam tomorrow 3pm !p1 #Personal ~30m” in the add bar. Every part you type becomes a chip before you press Enter.'],
      ['zap', 'Let Ordo rank the day', 'Focus sorts everything by an explainable score. Plan my day fills your capacity with the best next tasks.'],
      ['command', 'Fly with the keyboard', 'Ctrl+K opens the command palette. N adds a task, 1–8 switch views, and ? lists every shortcut.'],
    ];
    return `<div class="card tips">${tips.map(([ic, title, text]) => `<div class="tip"><div class="tip-icon">${icon(ic)}</div><div><strong>${title}</strong>${text}</div></div>`).join('')}<button class="icon-btn icon-btn-xs tips-close" data-action="dismiss-tips" title="Dismiss tips">${icon('x')}</button></div>`;
  }

  function renderToday() {
    const open = openTasks();
    const t = today();
    const overdue = sorted(open.filter((x) => x.due_date && x.due_date < t));
    const dueToday = sorted(open.filter((x) => x.due_date === t));
    const planned = sorted(open.filter((x) => isPlannedToday(x) && !(x.due_date && x.due_date <= t)));
    const completedToday = doneTasks().filter((x) => localDate(x.completed_at) === t).sort((a, b) => new Date(b.completed_at) - new Date(a.completed_at));
    const all = [...overdue, ...dueToday, ...planned];
    let html = todayHero(all, completedToday) + tipsCard();
    if (!all.length) {
      html += emptyState({ icon: 'coffee', title: 'Nothing planned for today', text: 'Add a task above, or let Ordo pick your best next tasks from the backlog.', cta: '<button class="btn btn-primary" data-action="plan-day">Plan my day</button>' });
    } else {
      html += section('Overdue', overdue, { danger: true });
      html += section('Due today', dueToday);
      html += section('Planned for today', planned, { hidePlan: true });
    }
    if (completedToday.length) {
      const show = S.showDone.today;
      html += `<div class="section"><div class="section-header"><h3>Completed today</h3><span class="n">${completedToday.length}</span><span class="spacer"></span><button class="toggle ${show ? 'open' : ''}" data-action="toggle-done" data-key="today">${icon('chevron-right')}${show ? 'Hide' : 'Show'}</button></div>${show ? listCard(completedToday, { score: false }) : ''}</div>`;
    }
    return html;
  }

  function renderFocus() {
    const list = sorted(openTasks(), 'score');
    let html = `<div class="card info-card">${icon('zap')}<div><p><strong>Ranked by focus score.</strong> Ordo combines what you told it about each task into one number so the top of this list is always the best thing to do next. Click a score to see why.</p>
      <div class="factors"><span class="factor">Priority</span><span class="factor">Deadline proximity</span><span class="factor">Planned for today</span><span class="factor">Starred</span><span class="factor">Waiting time</span><span class="factor">Quick wins</span><span class="factor">Momentum</span></div></div></div>`;
    if (!list.length) return html + emptyState({ icon: 'sparkles', title: 'Nothing to focus on', text: 'Your backlog is empty. Enjoy it, or add something above.' });
    html += toolbar(list.length);
    html += listCard(list, { ranked: true });
    return html;
  }

  function renderUpcoming() {
    const open = openTasks();
    const t = today();
    const dateOf = (x) => [x.due_date, x.scheduled].filter((d) => d && d > t).sort()[0];
    const future = open.filter((x) => dateOf(x));
    let html = '';
    for (let i = 1; i <= 7; i++) {
      const day = addDays(t, i);
      const list = sorted(future.filter((x) => dateOf(x) === day));
      html += `<div class="day-group"><div class="day-header"><h3>${i === 1 ? 'Tomorrow' : weekdayName(day)}</h3><span>${fmtDay(day, { day: 'numeric', month: 'short' })}</span></div>
        ${list.length ? listCard(list) : '<div class="day-empty">Nothing due</div>'}</div>`;
    }
    const later = sorted(future.filter((x) => daysBetween(t, dateOf(x)) > 7), 'due');
    if (later.length) html += section('Later', later);
    const undated = sorted(open.filter((x) => !x.due_date && !x.scheduled));
    if (undated.length) html += section('No date', undated, { extra: '<span class="spacer"></span><span class="muted small">Give these a date or plan them in Focus</span>' });
    return html;
  }

  function renderMatrix() {
    const open = openTasks();
    const q = {
      q1: { title: 'Do first', desc: 'Important & urgent', icon: 'flame' },
      q2: { title: 'Schedule', desc: 'Important, not urgent', icon: 'calendar' },
      q3: { title: 'Do quickly', desc: 'Urgent, not important', icon: 'zap' },
      q4: { title: 'Someday', desc: 'Neither urgent nor important', icon: 'coffee' },
    };
    return `<div class="matrix">${Object.entries(q).map(([k, v]) => {
      const list = sorted(open.filter((x) => x.quadrant === k), 'score');
      return `<div class="card quadrant ${k}" data-quadrant="${k}">
        <div class="quadrant-header"><span class="q-badge">${k.toUpperCase()}</span><div><h3>${v.title}</h3><p>${v.desc}</p></div><span class="n">${list.length}</span></div>
        <div class="quadrant-body">${list.length ? list.map((t, i) => taskRow(t, { index: i, draggable: true, score: false, reasons: false, showPriority: true, hidePlan: true })).join('') : `<div class="empty"><div class="empty-icon">${icon(v.icon)}</div><p class="small">Drop tasks here</p></div>`}</div>
      </div>`;
    }).join('')}</div>`;
  }

  function renderInbox() {
    const list = sorted(openTasks().filter((t) => !t.project_id));
    if (!list.length) return emptyState({ icon: 'inbox', title: 'Inbox zero', text: 'Every open task belongs to a project. Tasks added without <code>#project</code> land here.' });
    return toolbar(list.length) + listCard(list);
  }

  function renderCompleted() {
    const list = doneTasks().sort((a, b) => new Date(b.completed_at) - new Date(a.completed_at));
    if (!list.length) return emptyState({ icon: 'check-circle', title: 'Nothing completed yet', text: 'Finished tasks show up here, grouped by day.' });
    const groups = new Map();
    list.forEach((t) => { const d = localDate(t.completed_at); if (!groups.has(d)) groups.set(d, []); groups.get(d).push(t); });
    return Array.from(groups.entries()).map(([d, items]) => `<div class="day-group"><div class="day-header ${d === today() ? 'today' : ''}"><h3>${d === today() ? 'Today' : d === addDays(today(), -1) ? 'Yesterday' : fmtDay(d)}</h3><span>${items.length} done</span></div>${listCard(items, { score: false })}</div>`).join('');
  }

  function renderTrash() {
    const list = trashedTasks().sort((a, b) => new Date(b.deleted_at) - new Date(a.deleted_at));
    if (!list.length) return emptyState({ icon: 'trash', title: 'Trash is empty', text: 'Deleted tasks stay here until you empty the trash, so mistakes are cheap.' });
    return `<div class="row mb-16" style="justify-content:flex-end"><button class="btn btn-danger btn-sm" data-action="empty-trash">${icon('trash')}Empty trash</button></div>` + listCard(list, { score: false, reasons: false });
  }

  function renderProject() {
    const p = projectById(S.view.id);
    if (!p) return emptyState({ icon: 'folder', title: 'Project not found', text: 'It may have been deleted.', cta: '<button class="btn" data-action="go" data-view="today">Go to Today</button>' });
    const open = sorted(openTasks().filter((t) => t.project_id === p.id));
    const done = doneTasks().filter((t) => t.project_id === p.id).sort((a, b) => new Date(b.completed_at) - new Date(a.completed_at));
    const total = open.length + done.length;
    const pct = total ? Math.round((done.length / total) * 100) : 0;
    const est = open.reduce((a, t) => a + (t.estimate_minutes || 0), 0);
    const overdue = open.filter((t) => t.due_status === 'overdue').length;
    let html = `<div class="card project-hero"><div class="swatch" style="background:${esc(p.color)}"></div>
      <div class="ph-text"><strong>${esc(p.name)}</strong><p>${esc(p.description || 'No description yet')}</p><div class="progress"><i style="width:${pct}%;background:${esc(p.color)}"></i></div></div>
      <div class="ph-stat"><strong>${pct}%</strong>${done.length}/${total} done</div>
      <div class="ph-stat"><strong>${fmtMinutes(est)}</strong>remaining est.</div>
      ${overdue ? `<div class="ph-stat"><strong class="text-danger">${overdue}</strong>overdue</div>` : ''}
      <button class="btn btn-sm" data-action="edit-project" data-id="${p.id}">${icon('edit')}Edit</button></div>`;
    if (!open.length) html += emptyState({ icon: 'check-circle', title: total ? 'Project complete' : 'No tasks yet', text: total ? 'Every task in this project is done.' : 'Add the first task above.' });
    else html += toolbar(open.length) + listCard(open, { hideProject: true });
    if (done.length) {
      const show = S.showDone[p.id];
      html += `<div class="section"><div class="section-header"><h3>Completed</h3><span class="n">${done.length}</span><span class="spacer"></span><button class="toggle ${show ? 'open' : ''}" data-action="toggle-done" data-key="${p.id}">${icon('chevron-right')}${show ? 'Hide' : 'Show'}</button></div>${show ? listCard(done, { hideProject: true, score: false }) : ''}</div>`;
    }
    return html;
  }

  function renderTag() {
    const list = sorted(openTasks().filter((t) => t.tags.includes(S.view.id)));
    if (!list.length) return emptyState({ icon: 'tag', title: 'No open tasks with this tag', text: 'Add <code>@tag</code> to a task in the quick-add bar.' });
    return toolbar(list.length) + listCard(list);
  }

  function renderSearch() {
    const list = S.searchResults || [];
    if (!list.length) return emptyState({ icon: 'search', title: 'No matches', text: 'Search looks at titles, notes, tags and subtasks.' });
    return listCard(list, { highlight: S.search });
  }

  // ------------------------------------------------------------------
  // Rendering: analytics
  // ------------------------------------------------------------------
  function niceCeil(v) {
    if (v <= 0) return 1;
    const p = Math.pow(10, Math.floor(Math.log10(v)));
    const n = v / p;
    const m = n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10;
    return m * p;
  }

  function columnChart(id, data, opts = {}) {
    const W = 520, H = 190, padL = 30, padR = 8, padT = 18, padB = 26;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    const max = niceCeil(Math.max(1, ...data.map((d) => d.value)));
    const slot = innerW / data.length;
    const bw = Math.min(24, slot * 0.6);
    const ticks = [4, 5, 2, 1].find((n) => Number.isInteger(max / n)) || 4;
    const maxIdx = data.reduce((best, d, i) => (d.value > data[best].value ? i : best), 0);
    const grid = Array.from({ length: ticks + 1 }, (_, i) => {
      const y = padT + innerH - (innerH * i) / ticks;
      const v = (max * i) / ticks;
      return `<line x1="${padL}" x2="${W - padR}" y1="${y}" y2="${y}"/><text x="${padL - 6}" y="${y + 4}" text-anchor="end">${Number.isInteger(v) ? v : v.toFixed(1)}</text>`;
    }).join('');
    const bars = data.map((d, i) => {
      const x = padL + slot * i + (slot - bw) / 2;
      const h = (d.value / max) * innerH;
      const y = padT + innerH - h;
      const r = Math.min(4, h);
      const path = h > 0 ? `M${x},${padT + innerH} V${y + r} Q${x},${y} ${x + r},${y} H${x + bw - r} Q${x + bw},${y} ${x + bw},${y + r} V${padT + innerH} Z` : '';
      const label = i === maxIdx && d.value > 0 ? `<text class="value-label" x="${x + bw / 2}" y="${y - 5}" text-anchor="middle">${d.value}</text>` : '';
      const every = data.length > 8 ? 2 : 1;
      const xl = i % every === 0 || i === data.length - 1 ? `<text x="${x + bw / 2}" y="${H - 8}" text-anchor="middle">${esc(d.label)}</text>` : '';
      return `<path class="bar" d="${path}"/>${label}${xl}<rect class="bar hit" x="${padL + slot * i}" y="${padT}" width="${slot}" height="${innerH}" data-tip="${esc(d.tip || `${d.label}: ${d.value}`)}"/>`;
    }).join('');
    const table = S.tableView[id] ? `<table class="data-table"><tr><th>${esc(opts.xLabel || 'Label')}</th><th class="num">${esc(opts.yLabel || 'Value')}</th></tr>${data.map((d) => `<tr><td>${esc(d.long || d.label)}</td><td class="num">${d.value}</td></tr>`).join('')}</table>` : '';
    return `<svg class="chart" viewBox="0 0 ${W} ${H}" role="img" aria-label="${esc(opts.aria || '')}"><g class="grid">${grid}</g><line class="axis" x1="${padL}" x2="${W - padR}" y1="${padT + innerH}" y2="${padT + innerH}"/>${bars}</svg>
      <button class="table-toggle" data-action="toggle-table" data-id="${id}">${S.tableView[id] ? 'Hide data table' : 'Show data table'}</button>${table}`;
  }

  function hbars(rows, opts = {}) {
    const max = Math.max(1, ...rows.map((r) => r.max ?? r.value));
    return rows.map((r) => `<div class="hbar-row"><div class="hl" title="${esc(r.label)}"><i class="dot" style="background:${esc(r.color)}"></i><span>${esc(r.label)}</span></div>
      <div class="track" data-tip="${esc(r.tip || `${r.label}: ${r.value}`)}"><i style="width:${Math.round(((r.value) / (r.max ?? max)) * 100)}%;background:${esc(r.color)}"></i></div>
      <div class="hv">${esc(r.display ?? r.value)}</div></div>`).join('') + (opts.legend || '');
  }

  function statTile(label, value, sub, ic, subCls = '') {
    return `<div class="card stat-tile"><div class="label">${icon(ic)}${esc(label)}</div><div class="value">${value}</div>${sub ? `<div class="sub ${subCls}">${sub}</div>` : ''}</div>`;
  }

  function renderAnalytics() {
    const a = S.analytics;
    if (!a) return '<div class="muted">Loading…</div>';
    const yesterday = a.daily[a.daily.length - 2] ? a.daily[a.daily.length - 2].completed : 0;
    const delta = a.completed_today - yesterday;
    const est = a.estimate;
    const accuracy = est.tasks ? Math.round((est.spent_minutes / Math.max(1, est.estimated_minutes)) * 100) : null;
    let html = `<div class="stats-grid">
      ${statTile('Completed today', a.completed_today, delta === 0 ? 'same as yesterday' : `${delta > 0 ? '+' : ''}${delta} vs yesterday`, 'check-circle', delta > 0 ? 'good' : delta < 0 ? 'bad' : '')}
      ${statTile('This week', a.completed_week, `${a.completed_total} all time`, 'calendar')}
      ${statTile('Current streak', `${a.streak_days}d`, `best ${a.best_streak}d`, 'flame', a.streak_days >= 3 ? 'good' : '')}
      ${statTile('On-time rate', a.on_time_rate == null ? '—' : `${a.on_time_rate}%`, 'last 30 days, tasks with deadlines', 'target', a.on_time_rate >= 80 ? 'good' : a.on_time_rate != null && a.on_time_rate < 50 ? 'bad' : '')}
      ${statTile('Focus time', fmtMinutes(a.focus_minutes_week), `${a.focus_sessions_week} session${a.focus_sessions_week === 1 ? '' : 's'} this week`, 'clock')}
      ${statTile('Overdue', a.overdue, `${a.open_total} open · ${a.due_today} due today`, 'alert', a.overdue ? 'bad' : 'good')}
    </div>
    <div class="charts-grid">
      <div class="card chart-card"><h3>Completions, last 14 days</h3><p class="chart-sub">Tasks marked done per day</p>
        ${columnChart('daily', a.daily.map((d) => ({ label: d.label.split(' ')[1] || d.label, long: fmtDay(d.date), value: d.completed, tip: `${fmtDay(d.date)}: ${d.completed} done, ${d.created} added` })), { aria: 'Completed tasks per day', xLabel: 'Day', yLabel: 'Completed' })}</div>
      <div class="card chart-card"><h3>Most productive weekdays</h3><p class="chart-sub">Completions by weekday, last 8 weeks</p>
        ${columnChart('weekday', a.by_weekday.map((d) => ({ label: d.label, value: d.completed })), { aria: 'Completions by weekday', xLabel: 'Weekday', yLabel: 'Completed' })}</div>
      <div class="card chart-card"><h3>Open tasks by priority</h3><p class="chart-sub">Where your backlog sits on the importance scale</p>
        ${hbars(a.by_priority.map((p) => ({ label: `${p.priority.toUpperCase()} ${p.label}`, value: p.open, color: `var(--${p.priority})`, display: `${p.open}`, tip: `${p.label}: ${p.open} open, ${p.done} done` })))}</div>
      <div class="card chart-card"><h3>Projects</h3><p class="chart-sub">Completion progress per project</p>
        ${hbars(a.by_project.map((p) => ({ label: p.name, value: p.done, max: Math.max(1, p.open + p.done), color: p.color, display: `${p.done}/${p.open + p.done}`, tip: `${p.name}: ${p.done} done, ${p.open} open${p.overdue ? `, ${p.overdue} overdue` : ''}` })))}</div>
      <div class="card chart-card"><h3>Estimates vs. reality</h3><p class="chart-sub">Completed tasks with both an estimate and logged focus time</p>
        ${est.tasks ? `${hbars([
          { label: 'Estimated', value: est.estimated_minutes, max: Math.max(est.estimated_minutes, est.spent_minutes), color: 'var(--p4)', display: fmtMinutes(est.estimated_minutes) },
          { label: 'Actual', value: est.spent_minutes, max: Math.max(est.estimated_minutes, est.spent_minutes), color: 'var(--chart-1)', display: fmtMinutes(est.spent_minutes) },
        ])}<p class="small muted mt-8">Across ${est.tasks} task${est.tasks === 1 ? '' : 's'}, work took <strong>${accuracy}%</strong> of the estimated time${accuracy > 115 ? ' — pad your estimates a little.' : accuracy < 85 ? ' — you are faster than you think.' : ' — nicely calibrated.'}</p>` : '<p class="small muted">Run focus sessions on estimated tasks to see how accurate you are.</p>'}</div>
      <div class="card chart-card"><h3>Cycle time</h3><p class="chart-sub">Average age of a task when completed, last 30 days</p>
        <div class="value" style="font-size:30px;font-weight:700;letter-spacing:-0.02em">${a.avg_completion_days == null ? '—' : `${a.avg_completion_days} days`}</div>
        <p class="small muted mt-8">${a.planned_today} task${a.planned_today === 1 ? '' : 's'} planned for today · ${a.focus_minutes_total ? `${fmtMinutes(a.focus_minutes_total)} of focus logged all time` : 'no focus time logged yet'}</p></div>
    </div>`;
    return html;
  }

  // ------------------------------------------------------------------
  // Rendering: detail panel
  // ------------------------------------------------------------------
  function recurrenceKey(r) {
    if (!r) return '';
    if (r.kind === 'daily') return r.interval > 1 ? `daily:${r.interval}` : 'daily';
    if (r.kind === 'weekdays') return 'weekdays';
    if (r.kind === 'weekly') return r.interval === 2 ? 'biweekly' : r.interval > 2 ? `weekly:${r.interval}` : 'weekly';
    if (r.kind === 'monthly') return r.interval > 1 ? `monthly:${r.interval}` : 'monthly';
    if (r.kind === 'yearly') return 'yearly';
    return '';
  }
  function recurrenceFromKey(key, t) {
    if (!key) return null;
    const [kind, n] = key.split(':');
    const interval = Number(n) || (kind === 'biweekly' ? 2 : 1);
    const weekday = t.due_date ? (parseDate(t.due_date).getDay() + 6) % 7 : null;
    switch (kind) {
      case 'daily': return { kind: 'daily', interval, weekday: null };
      case 'weekdays': return { kind: 'weekdays', interval: 1, weekday: null };
      case 'weekly': case 'biweekly': return { kind: 'weekly', interval, weekday };
      case 'monthly': return { kind: 'monthly', interval, weekday: null };
      case 'yearly': return { kind: 'yearly', interval: 1, weekday: null };
      default: return null;
    }
  }

  function detailHTML(t) {
    const done = !!t.completed_at;
    const p = projectById(t.project_id);
    const ringLen = 2 * Math.PI * 26;
    const subDone = t.subtasks_done, subTotal = t.subtasks.length;
    const recKey = recurrenceKey(t.recurrence);
    const recOptions = [['', 'Does not repeat'], ['daily', 'Every day'], ['weekdays', 'Every weekday'], ['weekly', 'Every week'], ['biweekly', 'Every 2 weeks'], ['monthly', 'Every month'], ['yearly', 'Every year']];
    if (recKey && !recOptions.some(([k]) => k === recKey)) recOptions.push([recKey, t.recurrence_label]);
    return `<div class="detail-header">
      <button class="icon-btn" data-action="close-detail" title="Close (Esc)">${icon('x')}</button>
      <span class="spacer"></span>
      ${t.deleted_at ? `<button class="btn btn-sm" data-action="restore">${icon('restore')}Restore</button><button class="btn btn-sm btn-danger" data-action="purge">Delete forever</button>` : `
      <button class="icon-btn ${t.starred ? 'on' : ''}" data-action="star" title="Star">${icon('star')}</button>
      <button class="icon-btn" data-action="start-focus" title="Start focus session">${icon('play')}</button>
      <button class="icon-btn" data-action="duplicate" title="Duplicate">${icon('copy')}</button>
      <button class="icon-btn danger" data-action="trash" title="Move to trash">${icon('trash')}</button>`}
    </div>
    <div class="detail-body">
      <div class="detail-titlebar prio-${t.priority}">
        <button class="check big ${done ? 'done' : ''}" data-action="toggle-complete" title="${done ? 'Reopen' : 'Complete'}">${icon('check')}</button>
        <textarea class="detail-title ${done ? 'done' : ''}" data-field="title" rows="1" spellcheck="false">${esc(t.title)}</textarea>
      </div>
      ${!done && !t.deleted_at ? `<div class="card score-card">
        <div class="score-ring"><svg viewBox="0 0 62 62"><circle class="track" cx="31" cy="31" r="26"/><circle class="fill" cx="31" cy="31" r="26" stroke-dasharray="${ringLen}" stroke-dashoffset="${ringLen * (1 - t.score / 100)}"/></svg><div class="num">${t.score}</div></div>
        <div class="score-text"><strong>Focus score</strong><div class="q">${quadrantLabel(t.quadrant)} · ${t.important ? 'important' : 'not important'}, ${t.urgent ? 'urgent' : 'not urgent'}</div>
        <div class="reasons">${t.reasons.map((r) => `<span class="reason">${esc(r)}</span>`).join('')}</div></div>
      </div>` : done ? `<div class="card time-card">${icon('check-circle')}<div class="tc-text"><strong>Completed ${esc(fmtDateTime(t.completed_at))}</strong><span>Reopen it with the circle above</span></div></div>` : ''}
      <div class="detail-grid">
        <div class="field"><label>Project</label>
          <select class="input" data-field="project_id"><option value="">Inbox (no project)</option>${S.data.projects.map((pr) => `<option value="${pr.id}" ${pr.id === t.project_id ? 'selected' : ''}>${esc(pr.name)}</option>`).join('')}</select></div>
        <div class="field"><label>Estimate</label>
          <div class="field-row"><input class="input" type="number" min="0" step="5" data-field="estimate_minutes" value="${t.estimate_minutes || ''}" placeholder="minutes"></div>
          <div class="chips">${[15, 30, 60, 120].map((m) => `<button class="chip-btn ${t.estimate_minutes === m ? 'active' : ''}" data-action="set-estimate" data-value="${m}">${fmtMinutes(m)}</button>`).join('')}</div></div>
        <div class="field span-2"><label>Priority</label>
          <div class="segmented">${PRIORITIES.map((pr) => `<button class="seg prio-${pr.id} ${t.priority === pr.id ? 'active' : ''}" data-action="set-priority" data-value="${pr.id}"><i class="dot" style="background:var(--${pr.id})"></i>${pr.label}</button>`).join('')}</div></div>
        <div class="field"><label>Due</label>
          <div class="field-row"><input class="input" type="date" data-field="due_date" value="${t.due_date || ''}"><input class="input" type="time" data-field="due_time" value="${t.due_time ? t.due_time.slice(0, 5) : ''}" style="width:120px"></div>
          <div class="chips">${[['today', 'Today'], ['tomorrow', 'Tomorrow'], ['next-week', 'Next week'], ['none', 'Clear']].map(([k, l]) => `<button class="chip-btn" data-action="set-due" data-value="${k}">${l}</button>`).join('')}</div></div>
        <div class="field"><label>Planned for</label>
          <div class="field-row"><input class="input" type="date" data-field="scheduled" value="${t.scheduled || ''}"></div>
          <div class="chips">${[['today', 'Today'], ['tomorrow', 'Tomorrow'], ['next-week', 'Next week'], ['none', 'Clear']].map(([k, l]) => `<button class="chip-btn" data-action="set-sched" data-value="${k}">${l}</button>`).join('')}</div></div>
        <div class="field"><label>Repeat</label>
          <select class="input" data-field="recurrence">${recOptions.map(([k, l]) => `<option value="${k}" ${k === recKey ? 'selected' : ''}>${esc(l)}</option>`).join('')}</select></div>
        <div class="field"><label>Tags</label>
          <div class="tags-editor">${t.tags.map((tag) => `<span class="tag-chip">#${esc(tag)}<button data-action="remove-tag" data-value="${esc(tag)}" title="Remove">${icon('x')}</button></span>`).join('')}<input id="tag-input" placeholder="Add tag…"></div></div>
        <div class="field span-2"><label>Notes</label><textarea class="input" data-field="notes" placeholder="Details, links, context…">${esc(t.notes)}</textarea></div>
      </div>
      <div class="subtasks">
        <div class="field-label">Checklist ${subTotal ? `· ${subDone}/${subTotal}` : ''}</div>
        ${subTotal ? `<div class="sub-progress"><i style="width:${Math.round((subDone / subTotal) * 100)}%"></i></div>` : ''}
        ${t.subtasks.map((s) => `<div class="sub-row" data-sid="${s.id}"><button class="sub-check ${s.done ? 'done' : ''}" data-action="sub-toggle">${icon('check')}</button><input class="sub-title ${s.done ? 'done' : ''}" data-field="sub-title" value="${esc(s.title)}"><button class="icon-btn icon-btn-xs sub-del" data-action="sub-delete" title="Remove">${icon('x')}</button></div>`).join('')}
        <div class="sub-add">${icon('plus')}<input id="sub-input" placeholder="Add a step and press Enter"></div>
      </div>
      <div class="card time-card">${icon('clock')}<div class="tc-text"><strong>${t.time_spent_minutes ? `${fmtMinutes(t.time_spent_minutes)} focused` : 'No focus time yet'}</strong><span>${t.estimate_minutes ? `Estimated ${fmtMinutes(t.estimate_minutes)}` : 'Add an estimate to plan your day'}${p ? ` · ${esc(p.name)}` : ''}</span></div>
        ${!done && !t.deleted_at ? `<button class="btn btn-sm" data-action="start-focus">${icon('play')}Focus</button>` : ''}</div>
      <div class="detail-meta"><span>Created ${esc(fmtDateTime(t.created_at))}</span><span>Updated ${esc(fmtDateTime(t.updated_at))}</span>${t.deleted_at ? `<span class="text-danger">In trash since ${esc(fmtDateTime(t.deleted_at))}</span>` : ''}</div>
    </div>`;
  }

  function renderDetail() {
    const el = $('#detail');
    const t = S.selectedId ? taskById(S.selectedId) : null;
    if (!t) { el.hidden = true; el.innerHTML = ''; S.selectedId = null; return; }
    const top = el.scrollTop;
    el.hidden = false;
    el.innerHTML = detailHTML(t);
    el.scrollTop = top;
    const ta = $('.detail-title', el);
    autosize(ta);
    if (S.focusAfter) { const f = $(S.focusAfter, el); if (f) f.focus(); S.focusAfter = null; }
  }
  function autosize(ta) { if (!ta) return; ta.style.height = 'auto'; ta.style.height = `${ta.scrollHeight}px`; }

  // ------------------------------------------------------------------
  // Rendering: bulk bar
  // ------------------------------------------------------------------
  function renderBulkBar() {
    const el = $('#bulk-bar');
    const n = S.selection.size;
    if (!n) { el.hidden = true; return; }
    el.hidden = false;
    const inTrash = S.view.type === 'trash';
    el.innerHTML = `<strong>${n} selected</strong>
      ${inTrash ? `<button class="btn btn-sm" data-bulk="restore">${icon('restore')}Restore</button><button class="btn btn-sm btn-danger" data-bulk="purge">Delete forever</button>` : `
      <button class="btn btn-sm" data-bulk="complete">${icon('check')}Complete</button>
      <button class="btn btn-sm" data-bulk="schedule" data-value="${today()}">${icon('target')}Plan today</button>
      <select class="input input-sm" data-bulk-select="priority" style="width:auto"><option value="">Priority…</option>${PRIORITIES.map((p) => `<option value="${p.id}">${p.short} ${p.label}</option>`).join('')}</select>
      <select class="input input-sm" data-bulk-select="project" style="width:auto"><option value="">Move to…</option><option value="__inbox">Inbox</option>${S.data.projects.map((p) => `<option value="${p.id}">${esc(p.name)}</option>`).join('')}</select>
      <button class="btn btn-sm btn-danger" data-bulk="delete">${icon('trash')}Delete</button>`}
      <span class="sep"></span>
      <button class="btn btn-ghost btn-sm" data-action="clear-selection">Clear</button>`;
  }

  async function bulk(action, value) {
    const ids = Array.from(S.selection);
    await run(async () => {
      const r = await api('POST', '/api/tasks/bulk', { ids, action, value: value === undefined ? null : value });
      S.selection.clear();
      await refresh();
      toast(`${r.affected} task${r.affected === 1 ? '' : 's'} updated`);
    });
  }

  // ------------------------------------------------------------------
  // Task actions
  // ------------------------------------------------------------------
  /** Opens a task in the detail panel, updating the list highlight in place (no rebuild, no flash). */
  function select(id) {
    S.selectedId = id;
    S.focusAfter = null;
    $$('#content .task').forEach((r) => r.classList.toggle('selected', r.dataset.id === id));
    renderDetail();
  }
  function closeDetail() {
    const el = $('#detail');
    const finish = () => {
      el.classList.remove('closing');
      S.selectedId = null;
      $$('#content .task.selected').forEach((r) => r.classList.remove('selected'));
      renderDetail();
    };
    if (el.hidden || !S.selectedId) { finish(); return; }
    el.classList.add('closing');
    setTimeout(finish, 150);
  }
  function startInlineEdit(titleEl, t) {
    const input = document.createElement('input');
    input.className = 'inline-edit';
    input.value = t.title;
    titleEl.replaceChildren(input);
    input.focus();
    input.select();
    let finished = false;
    const finish = async (save) => {
      if (finished) return;
      finished = true;
      const v = input.value.trim();
      if (save && v && v !== t.title) await patchTask(t.id, { title: v }); else renderContent();
    };
    input.addEventListener('keydown', (e) => { e.stopPropagation(); if (e.key === 'Enter') { e.preventDefault(); finish(true); } if (e.key === 'Escape') { e.preventDefault(); finish(false); } });
    input.addEventListener('blur', () => finish(true));
    input.addEventListener('click', (e) => e.stopPropagation());
  }
  function toggleSelect(id) {
    if (S.selection.has(id)) S.selection.delete(id); else S.selection.add(id);
    render();
  }

  async function patchTask(id, body) {
    return run(async () => { await api('PATCH', `/api/tasks/${id}`, body); await refresh(); });
  }

  async function toggleComplete(id, rowEl) {
    const t = taskById(id);
    if (!t) return;
    if (t.completed_at) {
      await run(async () => { await api('POST', `/api/tasks/${id}/reopen`); await refresh(); });
      return;
    }
    if (rowEl) { rowEl.classList.add('checking'); await sleep(280); rowEl.classList.add('completing'); }
    await run(async () => {
      const r = await api('POST', `/api/tasks/${id}/complete`);
      await sleep(rowEl ? 200 : 0);
      await refresh();
      const msg = r.next ? `Done. Next occurrence: ${relDay(r.next.due_date)}` : `Completed “${t.title.length > 40 ? `${t.title.slice(0, 40)}…` : t.title}”`;
      toast(msg, { action: { label: 'Undo', fn: async () => { await api('POST', `/api/tasks/${id}/reopen`); if (r.next) await api('DELETE', `/api/tasks/${r.next.id}/purge`); await refresh(); } } });
    });
  }

  async function trashTask(id) {
    const t = taskById(id);
    await run(async () => {
      await api('DELETE', `/api/tasks/${id}`);
      if (S.selectedId === id) S.selectedId = null;
      await refresh();
      toast(`Moved “${t ? (t.title.length > 40 ? `${t.title.slice(0, 40)}…` : t.title) : 'task'}” to trash`, { action: { label: 'Undo', fn: async () => { await api('POST', `/api/tasks/${id}/restore`); await refresh(); } } });
    });
  }

  async function purgeTask(id) {
    if (!(await confirmDialog({ title: 'Delete forever?', message: 'This cannot be undone.', confirmLabel: 'Delete forever', danger: true }))) return;
    await run(async () => { await api('DELETE', `/api/tasks/${id}/purge`); if (S.selectedId === id) S.selectedId = null; await refresh(); });
  }

  async function planDay() {
    await run(async () => {
      const r = await api('POST', '/api/plan-day', {});
      await refresh();
      if (!r.planned.length) toast(r.skipped_for_capacity ? 'Your day is already at capacity. Finish something first, or raise the capacity in Settings.' : 'Nothing left to plan. Your backlog is clear.');
      else toast(`Planned ${r.planned.length} task${r.planned.length === 1 ? '' : 's'} · ${fmtMinutes(r.planned_minutes + r.already_planned_minutes)} of ${fmtMinutes(r.capacity_minutes)}${r.skipped_for_capacity ? ` · ${r.skipped_for_capacity} did not fit` : ''}`);
      if (S.view.type !== 'today') go({ type: 'today', id: null });
    });
  }

  function dateForKey(key) {
    if (key === 'today') return today();
    if (key === 'tomorrow') return addDays(today(), 1);
    if (key === 'next-week') { const d = parseDate(today()); const delta = ((8 - d.getDay()) % 7) || 7; return addDays(today(), delta); }
    return null;
  }

  async function handleAction(action, el, e) {
    const row = el.closest('.task');
    const id = row ? row.dataset.id : (S.selectedId || el.dataset.id);
    switch (action) {
      case 'go': {
        const view = el.dataset.view;
        go(view === 'project' || view === 'tag' ? { type: view, id: el.dataset.id } : { type: view, id: null });
        break;
      }
      case 'toggle-sidebar': {
        const app = $('#app');
        if (window.innerWidth <= 900) app.classList.toggle('sidebar-open');
        else { app.classList.toggle('sidebar-collapsed'); localStorage.setItem('ordo.sidebar', app.classList.contains('sidebar-collapsed') ? 'collapsed' : ''); }
        break;
      }
      case 'new-task': focusQuickAdd(); break;
      case 'new-project': openProjectModal(); break;
      case 'edit-project': e.stopPropagation(); openProjectModal(el.dataset.id); break;
      case 'open-settings': openSettings(); break;
      case 'open-palette': openPalette(); break;
      case 'open-syntax-help': openSyntaxHelp(); break;
      case 'ai-normalize': aiNormalize(); break;
      case 'toggle-theme': toggleTheme(); break;
      case 'close-detail': closeDetail(); break;
      case 'dismiss-tips': localStorage.setItem('ordo.tipsDismissed', '1'); renderContent(); toast('Tips hidden. Press ? any time for shortcuts.'); break;
      case 'defer': {
        const t = taskById(id); if (!t) break;
        const tomorrow = addDays(today(), 1);
        const before = { due_date: t.due_date, scheduled: t.scheduled };
        const patch = t.due_date && t.due_date <= today() ? { due_date: tomorrow, scheduled: t.scheduled ? tomorrow : null } : { scheduled: tomorrow };
        await patchTask(id, patch);
        toast('Moved to tomorrow', { action: { label: 'Undo', fn: async () => { await api('PATCH', `/api/tasks/${id}`, before); await refresh(); } } });
        break;
      }
      case 'clear-selection': S.selection.clear(); render(); break;
      case 'toggle-select': toggleSelect(id); break;
      case 'toggle-complete': toggleComplete(id, row); break;
      case 'toggle-reasons': if (S.expanded.has(id)) S.expanded.delete(id); else S.expanded.add(id); renderContent(); break;
      case 'toggle-done': S.showDone[el.dataset.key] = !S.showDone[el.dataset.key]; renderContent(); break;
      case 'toggle-table': S.tableView[el.dataset.id] = !S.tableView[el.dataset.id]; renderContent(); break;
      case 'set-sort': break;
      case 'star': { const t = taskById(id); if (t) patchTask(id, { starred: !t.starred }); break; }
      case 'plan-today': patchTask(id, { scheduled: today() }); break;
      case 'trash': trashTask(id); break;
      case 'restore': await run(async () => { await api('POST', `/api/tasks/${id}/restore`); await refresh(); toast('Task restored'); }); break;
      case 'purge': purgeTask(id); break;
      case 'duplicate': await run(async () => { const t = await api('POST', `/api/tasks/${id}/duplicate`); await refresh(); select(t.id); toast('Task duplicated'); }); break;
      case 'start-focus': startFocus(id); break;
      case 'plan-day': planDay(); break;
      case 'empty-trash': if (await confirmDialog({ title: 'Empty trash?', message: `${trashedTasks().length} tasks will be deleted forever.`, confirmLabel: 'Empty trash', danger: true })) await run(async () => { await api('POST', '/api/trash/empty'); S.selectedId = null; await refresh(); }); break;
      case 'set-priority': patchTask(id, { priority: el.dataset.value }); break;
      case 'set-estimate': patchTask(id, { estimate_minutes: Number(el.dataset.value) }); break;
      case 'set-due': patchTask(id, { due_date: dateForKey(el.dataset.value) }); break;
      case 'set-sched': patchTask(id, { scheduled: dateForKey(el.dataset.value) }); break;
      case 'remove-tag': { const t = taskById(id); if (t) patchTask(id, { tags: t.tags.filter((x) => x !== el.dataset.value) }); break; }
      case 'sub-toggle': { const t = taskById(id); const sid = el.closest('.sub-row').dataset.sid; const s = t && t.subtasks.find((x) => x.id === sid); if (s) await run(async () => { await api('PATCH', `/api/tasks/${id}/subtasks/${sid}`, { done: !s.done }); await refresh(); }); break; }
      case 'sub-delete': { const sid = el.closest('.sub-row').dataset.sid; await run(async () => { await api('DELETE', `/api/tasks/${id}/subtasks/${sid}`); await refresh(); }); break; }
      default: break;
    }
  }

  // ------------------------------------------------------------------
  // Quick add & parse preview
  // ------------------------------------------------------------------
  function focusQuickAdd() {
    if ($('#quick-add-wrap').hidden) go({ type: 'today', id: null });
    setTimeout(() => { const i = $('#quick-add'); i.focus(); i.select(); }, 0);
  }
  function quickContext() {
    const v = S.view;
    const ctx = {};
    if (v.type === 'project') ctx.project_id = v.id;
    if (v.type === 'today') ctx.scheduled = today();
    return ctx;
  }
  let previewTimer = null;
  async function updatePreview() {
    const q = $('#quick-add').value.trim();
    const box = $('#parse-preview');
    if (!q) { box.hidden = true; box.innerHTML = ''; return; }
    try {
      const text = S.view.type === 'tag' ? `${q} @${S.view.id}` : q;
      const p = await api('GET', `/api/parse?q=${encodeURIComponent(text)}`);
      const kindIcon = { project: 'folder', tag: 'tag', priority: 'flag', date: 'calendar', time: 'clock', estimate: 'clock', recurrence: 'repeat', star: 'star', planned: 'target', step: 'list', notes: 'info' };
      const chips = [`<span class="parse-chip kind-title">${icon('edit')}${p.title ? esc(p.title) : '<span class="muted">Untitled</span>'}</span>`];
      p.tokens.forEach((tk) => chips.push(`<span class="parse-chip kind-${tk.kind}">${icon(kindIcon[tk.kind] || 'info')}${esc(tk.value)}${tk.kind === 'project' && p.project_is_new ? '<span class="muted">(new)</span>' : ''}</span>`));
      if (!p.tokens.some((t) => t.kind === 'project') && quickContext().project_id) chips.push(`<span class="parse-chip kind-project">${icon('folder')}${esc(projectById(quickContext().project_id)?.name || '')}</span>`);
      if (S.view.type === 'today' && !p.tokens.some((t) => t.kind === 'date' || t.kind === 'planned')) chips.push(`<span class="parse-chip kind-planned">${icon('target')}Planned today</span>`);
      box.innerHTML = chips.map((c, i) => c.replace('<span class="parse-chip', `<span style="--i:${i}" class="parse-chip`)).join('');
      box.hidden = false;
    } catch (_) { /* ignore preview failures */ }
  }
  // ------------------------------------------------------------------
  // On-device AI
  // ------------------------------------------------------------------
  async function aiStatus() { S.ai = await api('GET', '/api/ai/status'); return S.ai; }
  function fmtBytes(b) { return b >= 1e9 ? `${(b / 1e9).toFixed(2)} GB` : `${Math.round(b / 1e6)} MB`; }
  function aiReady() {
    const st = S.ai; if (!st) return false;
    const m = st.models.find((x) => x.id === st.model);
    return !!m && m.status.state === 'ready';
  }
  function shortText(s, n = 60) { return s.length > n ? `${s.slice(0, n)}…` : s; }

  /** Rewrites the add-bar text with the on-device model. Returns the API result or null. */
  async function aiNormalize({ silent = false } = {}) {
    const input = $('#quick-add');
    const original = input.value.trim();
    if (!original) { toast('Type a sentence first, then let the AI clean it up.'); return null; }
    const btn = $('.quick-add-ai');
    const box = $('#parse-preview');
    btn.classList.add('busy');
    input.readOnly = true;
    box.hidden = false;
    box.innerHTML = `<span class="parse-chip kind-ai">${icon('sparkles')}Thinking on this device…</span>`;
    try {
      const r = await api('POST', '/api/ai/normalize', { text: original });
      input.value = r.output;
      await updatePreview();
      if (!silent) {
        toast(`AI read it as “${shortText(r.output)}” · ${(r.elapsed_ms / 1000).toFixed(1)}s`, {
          action: { label: 'Undo', fn: async () => { input.value = original; await updatePreview(); input.focus(); } },
        });
      }
      return r;
    } catch (e) {
      toast(e.message, { type: 'error', duration: 8000, action: /not downloaded/i.test(e.message) ? { label: 'Open settings', fn: openSettings } : null });
      await updatePreview();
      return null;
    } finally {
      btn.classList.remove('busy');
      input.readOnly = false;
      input.focus();
    }
  }

  async function setAi(patch) {
    await run(async () => { const s = await api('PATCH', '/api/settings', patch); S.data.settings = s; renderTopbar(); });
  }

  function aiModelRow(mo, selected) {
    const st = mo.status;
    let status = '', action = '';
    if (st.state === 'missing') {
      status = `<span class="muted">Not downloaded · ${fmtBytes(mo.size_bytes)}</span>`;
      action = `<button class="btn btn-sm" data-ai="download" data-model="${mo.id}">${icon('download')}Download</button>`;
    } else if (st.state === 'downloading') {
      const pct = st.total ? Math.round((st.done / st.total) * 100) : 0;
      status = `<div class="progress"><i style="width:${pct}%"></i></div><span class="muted">Downloading ${fmtBytes(st.done)} of ${fmtBytes(st.total)} (${pct}%)</span>`;
    } else if (st.state === 'ready') {
      status = `<span class="text-success">Ready${st.loaded ? ' · in memory' : ''} · ${fmtBytes(mo.size_bytes)}</span>`;
      action = `<button class="btn btn-sm btn-danger" data-ai="remove" data-model="${mo.id}">Remove</button>`;
    } else {
      status = `<span class="text-danger">${esc(st.message || 'Download failed')}</span>`;
      action = `<button class="btn btn-sm" data-ai="download" data-model="${mo.id}">Retry</button>`;
    }
    return `<label class="ai-model ${mo.id === selected ? 'active' : ''}"><input type="radio" name="ai-model" value="${mo.id}" data-ai="select" ${mo.id === selected ? 'checked' : ''}>
      <div class="ai-model-text"><strong>${esc(mo.label)}</strong><span>${esc(mo.description)}</span>${status}</div><div>${action}</div></label>`;
  }

  async function renderAiPanel(m) {
    const panel = $('#ai-panel', m);
    if (!panel || !document.body.contains(panel)) return;
    let st;
    try { st = await aiStatus(); } catch (e) { panel.innerHTML = `<p class="text-danger small">${esc(e.message)}</p>`; return; }
    const s = settings();
    const tryOut = $('#ai-try-out', panel);
    const keepOut = tryOut && !tryOut.hidden ? tryOut.innerHTML : '';
    const tryVal = $('#ai-try-input', panel)?.value || '';
    panel.innerHTML = `
      <div class="row" style="gap:18px;flex-wrap:wrap">
        <label class="row small"><input type="checkbox" data-ai="enabled" ${s.ai_enabled ? 'checked' : ''}> Show the AI button in the add bar</label>
        <label class="row small"><input type="checkbox" data-ai="auto" ${s.ai_auto ? 'checked' : ''} ${s.ai_enabled ? '' : 'disabled'}> Rewrite every new task automatically</label>
      </div>
      <div class="ai-models">${st.models.map((mo) => aiModelRow(mo, st.model)).join('')}</div>
      <div class="ai-try"><div class="field-label">Try it</div>
        <div class="row"><input class="input input-sm" id="ai-try-input" value="${esc(tryVal)}" placeholder="e.g. reply to sir tommorow arnd 3 in the evening, high prio"><button class="btn btn-sm btn-primary" data-ai="try" ${aiReady() ? '' : 'disabled'}>${icon('sparkles')}Run</button></div>
        <div id="ai-try-out" class="ai-try-out" ${keepOut ? '' : 'hidden'}>${keepOut}</div></div>
      <p class="field-help">Models live in <code>${esc(st.dir)}</code>. After first use the model stays in memory so later runs take well under a second, and it is unloaded by itself after 10 minutes without use; <button class="link" data-ai="unload">unload it now</button> to free the memory.</p>`;
    clearTimeout(S.aiPoll);
    if (st.models.some((x) => x.status.state === 'downloading')) S.aiPoll = setTimeout(() => renderAiPanel(m), 1000);
  }

  async function aiTry(m) {
    const input = $('#ai-try-input', m);
    const out = $('#ai-try-out', m);
    const text = input.value.trim();
    if (!text) { input.focus(); return; }
    out.hidden = false;
    out.innerHTML = `<span class="parse-chip kind-ai">${icon('sparkles')}Thinking…</span>`;
    try {
      const r = await api('POST', '/api/ai/normalize', { text });
      const chips = r.parsed.tokens.map((tk) => `<span class="parse-chip kind-${tk.kind}">${esc(tk.value)}</span>`).join('');
      out.innerHTML = `<div class="out-line">${esc(r.output)}</div><div class="chips"><span class="parse-chip kind-title">${esc(r.parsed.title || 'Untitled')}</span>${chips}</div><span class="muted small">${(r.elapsed_ms / 1000).toFixed(1)}s${r.load_ms ? ` (+${(r.load_ms / 1000).toFixed(1)}s to load the model)` : ''}</span>`;
    } catch (e) {
      out.innerHTML = `<span class="text-danger">${esc(e.message)}</span>`;
    }
  }

  async function submitQuickAdd() {
    const input = $('#quick-add');
    let text = input.value.trim();
    if (!text) return;
    let aiNote = '';
    if (settings().ai_enabled && settings().ai_auto) {
      const r = await aiNormalize({ silent: true });
      if (r && r.output !== text) { text = r.output; aiNote = ` · AI read it as “${shortText(r.output, 40)}”`; }
    }
    const ctx = quickContext();
    const body = { text: S.view.type === 'tag' ? `${text} @${S.view.id}` : text, ...ctx };
    await run(async () => {
      const r = await api('POST', '/api/tasks/quick', body);
      input.value = '';
      $('#parse-preview').hidden = true;
      await refresh();
      const t = r.task;
      const bits = [];
      if (t.due_date) bits.push(relDay(t.due_date));
      if (t.priority !== 'p3') bits.push(PRIORITIES.find((p) => p.id === t.priority).label);
      if (r.created_project) bits.push(`new project ${r.created_project.name}`);
      toast(`Added “${t.title.length > 36 ? `${t.title.slice(0, 36)}…` : t.title}”${bits.length ? ` · ${bits.join(', ')}` : ''}${aiNote}`, { action: { label: 'Open', fn: () => select(t.id) }, duration: aiNote ? 8000 : 5000 });
    });
  }

  // ------------------------------------------------------------------
  // Search
  // ------------------------------------------------------------------
  let searchTimer = null;
  async function doSearch(q) {
    S.search = q.trim();
    if (!S.search) { S.searchResults = null; render(); return; }
    S.searchResults = await run(() => api('GET', `/api/search?q=${encodeURIComponent(S.search)}`)) || [];
    S.animateNext = true;
    render();
  }

  // ------------------------------------------------------------------
  // Toasts
  // ------------------------------------------------------------------
  function toast(message, { type = 'info', action = null, duration = 5000 } = {}) {
    const box = $('#toasts');
    const el = document.createElement('div');
    el.className = `toast ${type}`;
    el.style.setProperty('--life', `${duration}ms`);
    el.innerHTML = `<span>${esc(message)}</span>${action ? `<button class="toast-action">${esc(action.label)}</button>` : ''}<button class="toast-x" title="Dismiss">${icon('x')}</button>`;
    const remove = () => { el.remove(); };
    $('.toast-x', el).addEventListener('click', remove);
    if (action) $('.toast-action', el).addEventListener('click', async () => { remove(); await run(action.fn); });
    box.appendChild(el);
    while (box.children.length > 3) box.firstChild.remove();
    setTimeout(remove, duration);
  }

  // ------------------------------------------------------------------
  // Modals
  // ------------------------------------------------------------------
  function openModal({ title, body, footer = '', wide = false, onMount }) {
    const m = $('#modal');
    m.hidden = false;
    m.innerHTML = `<div class="modal-box ${wide ? 'wide' : ''}" role="dialog" aria-modal="true">
      <div class="modal-head"><h2>${esc(title)}</h2><button class="icon-btn" data-modal="close" title="Close">${icon('x')}</button></div>
      <div class="modal-body">${body}</div>${footer ? `<div class="modal-foot">${footer}</div>` : ''}</div>`;
    if (onMount) onMount(m);
    const first = $('input, select, textarea, button.btn-primary', m);
    if (first) setTimeout(() => first.focus(), 0);
  }
  function closeModal() { const m = $('#modal'); m.hidden = true; m.innerHTML = ''; clearTimeout(S.aiPoll); }
  function modalOpen() { return !$('#modal').hidden; }

  function confirmDialog({ title, message, confirmLabel = 'Confirm', cancelLabel = 'Cancel', danger = false, extra = null }) {
    return new Promise((resolve) => {
      openModal({
        title,
        body: `<p style="margin:0">${esc(message)}</p>${extra || ''}`,
        footer: `<span class="spacer"></span><button class="btn" data-modal="close">${esc(cancelLabel)}</button><button class="btn ${danger ? 'btn-danger-solid' : 'btn-primary'}" id="confirm-ok">${esc(confirmLabel)}</button>`,
        onMount(m) {
          $('#confirm-ok', m).addEventListener('click', () => { closeModal(); resolve(true); });
          m.addEventListener('modal-closed', () => resolve(false), { once: true });
        },
      });
      setTimeout(() => $('#confirm-ok').focus(), 0);
    });
  }

  function openProjectModal(id) {
    const p = id ? projectById(id) : null;
    let color = p ? p.color : PROJECT_COLORS[S.data.projects.length % PROJECT_COLORS.length];
    openModal({
      title: p ? 'Edit project' : 'New project',
      body: `<div class="field"><label>Name</label><input class="input" id="pm-name" value="${esc(p ? p.name : '')}" placeholder="e.g. Product Launch"></div>
        <div class="field"><label>Color</label><div class="swatches" id="pm-swatches">${PROJECT_COLORS.map((c) => `<button class="swatch-btn ${c === color ? 'active' : ''}" data-color="${c}" style="background:${c}" title="${c}"></button>`).join('')}<label class="swatch-custom" title="Custom color"><input type="color" id="pm-custom" value="${esc(color)}"></label></div></div>
        <div class="field"><label>Description</label><input class="input" id="pm-desc" value="${esc(p ? p.description : '')}" placeholder="What is this project about?"></div>
        ${p ? `<div class="danger-zone"><div class="dz-row"><p><strong>Delete project</strong>Tasks can be kept in the inbox or moved to the trash.</p><button class="btn btn-sm btn-danger" id="pm-delete-keep">Delete, keep tasks</button><button class="btn btn-sm btn-danger" id="pm-delete-all">Delete with tasks</button></div></div>` : ''}`,
      footer: `<span class="spacer"></span><button class="btn" data-modal="close">Cancel</button><button class="btn btn-primary" id="pm-save">${p ? 'Save changes' : 'Create project'}</button>`,
      onMount(m) {
        $('#pm-swatches', m).addEventListener('click', (e) => {
          const b = e.target.closest('.swatch-btn'); if (!b) return;
          color = b.dataset.color; $$('.swatch-btn', m).forEach((x) => x.classList.toggle('active', x === b));
        });
        $('#pm-custom', m).addEventListener('input', (e) => { color = e.target.value; $$('.swatch-btn', m).forEach((x) => x.classList.remove('active')); });
        const save = async () => {
          const name = $('#pm-name', m).value.trim();
          if (!name) { $('#pm-name', m).focus(); return; }
          const body = { name, color, description: $('#pm-desc', m).value.trim() };
          await run(async () => {
            const saved = p ? await api('PATCH', `/api/projects/${p.id}`, body) : await api('POST', '/api/projects', body);
            closeModal();
            await refresh();
            if (!p) go({ type: 'project', id: saved.id });
            toast(p ? 'Project updated' : `Project “${saved.name}” created`);
          });
        };
        $('#pm-save', m).addEventListener('click', save);
        $('#pm-name', m).addEventListener('keydown', (e) => { if (e.key === 'Enter') save(); });
        if (p) {
          const del = async (mode) => {
            closeModal();
            if (!(await confirmDialog({ title: `Delete “${p.name}”?`, message: mode === 'delete' ? 'The project and all of its tasks will be moved to the trash.' : 'The project will be removed. Its tasks move to the inbox.', confirmLabel: 'Delete project', danger: true }))) return;
            await run(async () => { await api('DELETE', `/api/projects/${p.id}?mode=${mode}`); if (S.view.type === 'project' && S.view.id === p.id) go({ type: 'today', id: null }); await refresh(); toast('Project deleted'); });
          };
          $('#pm-delete-keep', m).addEventListener('click', () => del('keep'));
          $('#pm-delete-all', m).addEventListener('click', () => del('delete'));
        }
      },
    });
  }

  function openSettings() {
    const s = settings();
    openModal({
      title: 'Settings',
      wide: true,
      body: `<div class="modal-section"><h4>Profile</h4><div class="modal-grid">
          <div class="field"><label>Your name</label><input class="input" id="st-name" value="${esc(s.user_name)}"></div>
          <div class="field"><label>Workspace</label><input class="input" id="st-ws" value="${esc(s.workspace_name)}"></div></div></div>
        <div class="modal-section"><h4>Planning</h4><div class="modal-grid">
          <div class="field"><label>Daily capacity (hours)</label><input class="input" id="st-cap" type="number" min="0.5" max="24" step="0.5" value="${(s.daily_capacity_minutes / 60).toFixed(1).replace(/\\.0$/, '')}"><span class="field-help">How much focused work fits in your day. Plan my day fills up to this.</span></div>
          <div class="field"><label>Urgent window (days)</label><input class="input" id="st-urg" type="number" min="0" max="30" value="${s.urgent_window_days}"><span class="field-help">Tasks due within this many days count as urgent in the matrix.</span></div>
          <div class="field"><label>Focus session (minutes)</label><input class="input" id="st-pomo" type="number" min="1" max="180" value="${s.pomodoro_minutes}"></div>
          <div class="field"><label>Default estimate (minutes)</label><input class="input" id="st-est" type="number" min="5" max="480" step="5" value="${s.default_estimate_minutes}"><span class="field-help">Assumed for tasks without an estimate when planning.</span></div></div></div>
        <div class="modal-section"><h4>Appearance</h4><p class="field-help" style="margin:0 0 12px">Changes apply immediately.</p>
          <div class="field"><label>Theme</label><div class="seg-group" data-appearance-group="theme">${[['system', 'Match system'], ['light', 'Light'], ['dark', 'Dark']].map(([k, l]) => `<button class="seg ${appearance().theme === k ? 'active' : ''}" data-appearance="theme" data-value="${k}">${l}</button>`).join('')}</div></div>
          <div class="field mt-8"><label>Dark style</label><div class="style-cards" data-appearance-group="dark_style">${DARK_STYLES.map((d) => `<button class="style-card ${appearance().dark_style === d.id ? 'active' : ''}" data-appearance="dark_style" data-value="${d.id}">
              <div class="preview" style="background:${d.bg}"><i class="pv-side" style="background:${d.side}"></i><div class="pv-main"><i class="pv-accent"></i><i style="background:${d.card}"></i><i style="background:${d.card};width:70%"></i></div></div>
              <strong>${d.label}</strong><span>${d.desc}</span></button>`).join('')}</div></div>
          <div class="field mt-8"><label>Accent</label><div class="accent-swatches" data-appearance-group="accent">${ACCENTS.map((a) => `<button class="accent-swatch ${appearance().accent === a.id ? 'active' : ''}" data-appearance="accent" data-value="${a.id}" title="${a.label}" style="background:${a.swatch}">${icon('check')}</button>`).join('')}
            ${(() => { const cur = appearance().accent; const custom = isHex(cur); return `<label class="accent-swatch accent-custom ${custom ? 'active' : ''}" title="Custom colour" style="background:${custom ? cur : CUSTOM_WHEEL}"><input type="color" id="accent-picker" value="${custom ? cur : '#5b5bd6'}">${icon('check')}</label><input class="input input-sm" id="accent-hex" placeholder="#hex" maxlength="7" spellcheck="false" value="${custom ? cur : ''}" style="width:96px">`; })()}
          </div><span class="field-help">Pick a preset, use the colour wheel, or type a hex value. Very light or dark picks are adjusted so text stays readable.</span></div></div>
        <div class="modal-section"><h4>On-device AI</h4>
          <p class="field-help" style="margin:0 0 10px">A small language model runs on this computer to understand messy sentences and typos before the usual parsing. No internet after the one-time download, no API key, and nothing you type leaves your machine.</p>
          <div id="ai-panel"><span class="muted small">Loading…</span></div></div>
        <div class="modal-section"><h4>Data</h4>
          <div class="row" style="flex-wrap:wrap;gap:8px">
            <a class="btn btn-sm" href="/api/export" download>${icon('download')}Export JSON</a>
            <label class="btn btn-sm">${icon('upload')}Import JSON<input type="file" id="st-import" accept="application/json,.json" class="hidden-input"></label>
            <select class="input input-sm" id="st-import-mode" style="width:auto"><option value="merge">Merge with existing</option><option value="replace">Replace everything</option></select>
          </div>
          <p class="field-help mt-8">Stored locally in <code>${esc(S.data.data_path)}</code>. Nothing leaves your machine.</p></div>
        <div class="modal-section"><h4>Danger zone</h4><div class="danger-zone">
          <div class="dz-row"><p><strong>Empty trash</strong>Permanently delete ${trashedTasks().length} trashed task${trashedTasks().length === 1 ? '' : 's'}.</p><button class="btn btn-sm btn-danger" id="st-empty">Empty trash</button></div>
          <div class="dz-row"><p><strong>Reset demo workspace</strong>Replace all tasks and projects with the sample data.</p><button class="btn btn-sm btn-danger" id="st-reset">Reset</button></div>
          <div class="dz-row"><p><strong>Delete everything</strong>Start from a completely empty workspace.</p><button class="btn btn-sm btn-danger" id="st-clear">Delete all</button></div></div></div>
        <p class="small muted">Ordo v${esc(S.data.version)} · press <kbd>?</kbd> for keyboard shortcuts</p>`,
      footer: `<span class="spacer"></span><button class="btn" data-modal="close">Cancel</button><button class="btn btn-primary" id="st-save">Save settings</button>`,
      onMount(m) {
        $('#st-save', m).addEventListener('click', async () => {
          const body = {
            user_name: $('#st-name', m).value,
            workspace_name: $('#st-ws', m).value,
            daily_capacity_minutes: Math.round(parseFloat($('#st-cap', m).value || '6') * 60),
            urgent_window_days: parseInt($('#st-urg', m).value || '2', 10),
            pomodoro_minutes: parseInt($('#st-pomo', m).value || '25', 10),
            default_estimate_minutes: parseInt($('#st-est', m).value || '30', 10),
          };
          await run(async () => { await api('PATCH', '/api/settings', body); closeModal(); await refresh(); toast('Settings saved'); });
        });
        renderAiPanel(m);
        m.addEventListener('click', async (e) => {
          const b = e.target.closest('[data-ai]'); if (!b || b.tagName === 'INPUT') return;
          e.preventDefault();
          switch (b.dataset.ai) {
            case 'download': await run(async () => { await api('POST', '/api/ai/download', { model: b.dataset.model }); }); renderAiPanel(m); break;
            case 'remove': await run(async () => { await api('POST', '/api/ai/remove', { model: b.dataset.model }); toast('Model removed'); }); renderAiPanel(m); break;
            case 'unload': await run(async () => { await api('POST', '/api/ai/unload'); toast('Model unloaded from memory'); }); renderAiPanel(m); break;
            case 'try': aiTry(m); break;
            default: break;
          }
        });
        m.addEventListener('change', async (e) => {
          const el = e.target.closest('input[data-ai]'); if (!el) return;
          if (el.dataset.ai === 'enabled') await setAi({ ai_enabled: el.checked });
          if (el.dataset.ai === 'auto') await setAi({ ai_auto: el.checked });
          if (el.dataset.ai === 'select') await setAi({ ai_model: el.value });
          renderAiPanel(m);
        });
        m.addEventListener('keydown', (e) => { if (e.target.id === 'ai-try-input' && e.key === 'Enter') { e.preventDefault(); aiTry(m); } });
        const customSwatch = $('.accent-custom', m);
        m.addEventListener('click', (e) => {
          const b = e.target.closest('[data-appearance]'); if (!b) return;
          const key = b.dataset.appearance;
          $$(`[data-appearance="${key}"]`, m).forEach((x) => x.classList.toggle('active', x === b));
          if (key === 'accent') { customSwatch.classList.remove('active'); customSwatch.style.background = CUSTOM_WHEEL; $('#accent-hex', m).value = ''; }
          setAppearance({ [key]: b.dataset.value });
        });
        const setCustomAccent = (raw) => {
          let hex = (raw || '').trim().toLowerCase();
          if (!hex.startsWith('#')) hex = `#${hex}`;
          if (!isHex(hex)) { toast('Use a 6-digit hex colour like #3a86ff', { type: 'error' }); return; }
          $$('[data-appearance="accent"]', m).forEach((x) => x.classList.remove('active'));
          customSwatch.classList.add('active');
          customSwatch.style.background = hex;
          $('#accent-hex', m).value = hex;
          $('#accent-picker', m).value = hex;
          setAppearance({ accent: hex });
        };
        let pickTimer = null;
        $('#accent-picker', m).addEventListener('input', (e) => { clearTimeout(pickTimer); pickTimer = setTimeout(() => setCustomAccent(e.target.value), 80); });
        $('#accent-hex', m).addEventListener('change', (e) => { if (e.target.value.trim()) setCustomAccent(e.target.value); });
        $('#accent-hex', m).addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); setCustomAccent(e.target.value); } });
        $('#st-import', m).addEventListener('change', async (e) => {
          const file = e.target.files[0]; if (!file) return;
          const mode = $('#st-import-mode', m).value;
          await run(async () => {
            const data = JSON.parse(await file.text());
            if (mode === 'replace' && !(await confirmDialog({ title: 'Replace all data?', message: 'Everything currently in Ordo will be overwritten by the imported file.', confirmLabel: 'Replace', danger: true }))) return;
            const r = await api('POST', `/api/import?mode=${mode}`, data);
            closeModal(); await refresh();
            toast(`Imported. Workspace now has ${r.tasks} tasks in ${r.projects} projects.`);
          });
        });
        $('#st-empty', m).addEventListener('click', async () => { closeModal(); if (await confirmDialog({ title: 'Empty trash?', message: 'Trashed tasks will be deleted forever.', confirmLabel: 'Empty trash', danger: true })) await run(async () => { await api('POST', '/api/trash/empty'); await refresh(); toast('Trash emptied'); }); });
        $('#st-reset', m).addEventListener('click', async () => { closeModal(); if (await confirmDialog({ title: 'Reset to demo data?', message: 'All your tasks and projects will be replaced with the sample workspace.', confirmLabel: 'Reset', danger: true })) await run(async () => { await api('POST', '/api/reset-demo'); S.selectedId = null; await refresh(); toast('Demo workspace restored'); }); });
        $('#st-clear', m).addEventListener('click', async () => { closeModal(); if (await confirmDialog({ title: 'Delete everything?', message: 'All tasks, projects and focus history will be removed permanently.', confirmLabel: 'Delete all', danger: true })) await run(async () => { await api('POST', '/api/clear-all'); S.selectedId = null; await refresh(); toast('Workspace cleared'); }); });
      },
    });
  }

  function openShortcuts() {
    const rows = [
      ['New task', ['N']], ['Search', ['/']], ['Command palette', ['Ctrl', 'K']], ['Close / clear', ['Esc']],
      ['Today · Focus · Upcoming · Matrix', ['1', '2', '3', '4']], ['Inbox · Completed · Analytics · Trash', ['5', '6', '7', '8']],
      ['Next / previous task', ['J', 'K']], ['Complete selected task', ['C']], ['Star selected task', ['S']],
      ['Plan selected task for today', ['T']], ['Trash selected task', ['Del']], ['Start focus on selected task', ['F']],
      ['Toggle dark mode', ['D']], ['Multi-select a task', ['Shift', 'Click']], ['This help', ['?']],
    ];
    openModal({ title: 'Keyboard shortcuts', wide: true, body: `<div class="shortcut-list">${rows.map(([l, keys]) => `<div class="shortcut-row"><span>${l}</span><span class="keys">${keys.map((k) => `<kbd>${k}</kbd>`).join('')}</span></div>`).join('')}</div>` });
  }

  function openSyntaxHelp() {
    const rows = [
      ['<code>#Work</code> or <code>#Home Renovation</code>', 'File under a project (created if it does not exist)'],
      ['<code>@finance</code>', 'Add a tag'],
      ['<code>!p1</code> <code>!high</code> <code>!!!</code> <code>p2</code>', 'Priority: p1 critical, p2 high, p3 medium, p4 low'],
      ['<code>today</code> <code>tomorrow</code> <code>friday</code> <code>next monday</code>', 'Due dates in plain words'],
      ['<code>in 3 days</code> <code>in 2 weeks</code> <code>sep 15</code> <code>2026-10-01</code>', 'Relative and absolute dates'],
      ['<code>eow</code> <code>eom</code> <code>weekend</code> <code>tonight</code>', 'End of week, end of month, Saturday, 8pm today'],
      ['<code>5pm</code> <code>at 17:30</code> <code>noon</code>', 'Time of day'],
      ['<code>~45m</code> <code>1h30m</code> <code>2 hours</code> <code>in 10 mins</code> <code>for about 1h</code>', 'Time estimate (used by Plan my day)'],
      ['<code>every day</code> <code>every friday</code> <code>every 2 weeks</code> <code>monthly</code>', 'Recurring tasks'],
      ['<code>^tomorrow</code> <code>^next monday</code>', 'Planned-for date: the day you will work on it, separate from the deadline'],
      ['<code>+outline +draft the post</code>', 'Checklist steps'],
      ['<code>// any text</code>', 'Notes: everything after the double slash'],
      ['<code>hacking challenge project</code> <code>for SMEC Technologies</code>', 'Existing projects are recognised from plain words too. Single-word names need “project” after them, or a #'],
      ['<code>*</code>', 'Star the task'],
    ];
    openModal({ title: 'Quick-add syntax', wide: true, body: `<p class="small muted" style="margin:0">Type naturally. Everything Ordo recognises becomes a chip under the input before you press Enter.</p><table class="syntax-table">${rows.map(([a, b]) => `<tr><td>${a}</td><td>${b}</td></tr>`).join('')}</table><p class="small muted" style="margin:0">Example: <code>Send invoice to Acme tomorrow 5pm !p1 #Work @finance ~45m every friday ^today +draft +send // ask for PO number</code></p>` });
  }

  // ------------------------------------------------------------------
  // Command palette
  // ------------------------------------------------------------------
  function paletteItems(q) {
    const query = q.trim().toLowerCase();
    const items = [
      { group: 'Actions', label: 'New task', icon: 'plus', kbd: 'N', run: focusQuickAdd },
      { group: 'Actions', label: 'Plan my day', icon: 'sparkles', run: planDay },
      { group: 'Actions', label: 'New project', icon: 'folder', run: () => openProjectModal() },
      { group: 'Actions', label: 'Toggle dark mode', icon: 'moon', kbd: 'D', run: toggleTheme },
      { group: 'Actions', label: 'Switch dark style (soft / plain)', icon: 'layers', run: toggleDarkStyle },
      ...ACCENTS.map((a) => ({ group: 'Accent colour', label: `Accent: ${a.label}`, color: a.swatch, run: () => setAppearance({ accent: a.id }) })),
      { group: 'Actions', label: 'Settings', icon: 'settings', run: openSettings },
      { group: 'Actions', label: 'Keyboard shortcuts', icon: 'keyboard', kbd: '?', run: openShortcuts },
      { group: 'Actions', label: 'Quick-add syntax help', icon: 'help', run: openSyntaxHelp },
      { group: 'Actions', label: 'Export data as JSON', icon: 'download', run: () => { window.location.href = '/api/export'; } },
      ...VIEWS.map((v) => ({ group: 'Go to', label: v.label, icon: v.icon, kbd: v.key, run: () => go({ type: v.id, id: null }) })),
      ...S.data.projects.map((p) => ({ group: 'Projects', label: p.name, color: p.color, run: () => go({ type: 'project', id: p.id }) })),
      ...(S.data.tags || []).map((t) => ({ group: 'Tags', label: `#${t.name}`, icon: 'tag', run: () => go({ type: 'tag', id: t.name }) })),
    ];
    const score = (label) => { const l = label.toLowerCase(); if (!query) return 1; if (l.startsWith(query)) return 3; if (l.includes(query)) return 2; let i = 0; for (const ch of l) if (ch === query[i]) i++; return i === query.length ? 1 : 0; };
    const ranked = items.map((it) => ({ ...it, s: score(it.label) })).filter((it) => it.s > 0).sort((a, b) => b.s - a.s);
    if (query.length >= 2) {
      const tasks = openTasks().filter((t) => t.title.toLowerCase().includes(query)).slice(0, 8)
        .map((t) => ({ group: 'Tasks', label: t.title, icon: 'check-circle', sub: projectById(t.project_id)?.name || 'Inbox', run: () => select(t.id) }));
      ranked.push(...tasks);
    }
    return ranked.slice(0, 40);
  }
  function openPalette() {
    S.paletteOpen = true; S.paletteQuery = ''; S.paletteIndex = 0;
    const p = $('#palette');
    p.hidden = false;
    p.innerHTML = `<div class="palette-box"><div class="palette-input">${icon('search')}<input id="palette-input" placeholder="Type a command, view, project or task…" autocomplete="off"></div><div class="palette-list" id="palette-list"></div><div class="palette-foot"><span><kbd>↑↓</kbd> navigate</span><span><kbd>↵</kbd> select</span><span><kbd>Esc</kbd> close</span></div></div>`;
    renderPaletteList();
    const input = $('#palette-input');
    input.focus();
    input.addEventListener('input', () => { S.paletteQuery = input.value; S.paletteIndex = 0; renderPaletteList(); });
    input.addEventListener('keydown', (e) => {
      const items = paletteItems(S.paletteQuery);
      if (e.key === 'ArrowDown') { e.preventDefault(); S.paletteIndex = Math.min(items.length - 1, S.paletteIndex + 1); renderPaletteList(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); S.paletteIndex = Math.max(0, S.paletteIndex - 1); renderPaletteList(); }
      else if (e.key === 'Enter') { e.preventDefault(); const it = items[S.paletteIndex]; if (it) { closePalette(); it.run(); } }
      else if (e.key === 'Escape') { closePalette(); }
    });
  }
  function renderPaletteList() {
    const items = paletteItems(S.paletteQuery);
    const list = $('#palette-list');
    if (!items.length) { list.innerHTML = '<div class="palette-empty">No matches</div>'; return; }
    let html = '', lastGroup = null;
    items.forEach((it, i) => {
      if (it.group !== lastGroup) { html += `<div class="palette-group">${it.group}</div>`; lastGroup = it.group; }
      html += `<button class="palette-item ${i === S.paletteIndex ? 'active' : ''}" data-index="${i}">${it.color ? `<i class="dot" style="width:10px;height:10px;border-radius:50%;background:${esc(it.color)}"></i>` : icon(it.icon || 'arrow-right')}<span class="pi-text">${esc(it.label)}${it.sub ? ` <span class="pi-sub">· ${esc(it.sub)}</span>` : ''}</span>${it.kbd ? `<kbd>${it.kbd}</kbd>` : ''}</button>`;
    });
    list.innerHTML = html;
    const active = $('.palette-item.active', list);
    if (active) active.scrollIntoView({ block: 'nearest' });
  }
  function closePalette() { S.paletteOpen = false; const p = $('#palette'); p.hidden = true; p.innerHTML = ''; }

  // ------------------------------------------------------------------
  // Focus timer
  // ------------------------------------------------------------------
  const T = { taskId: null, total: 0, remaining: 0, running: false, last: 0, finished: false, tick: null };
  function saveTimer() { try { localStorage.setItem('ordo.timer', T.taskId ? JSON.stringify({ taskId: T.taskId, total: T.total, remaining: T.remaining, running: T.running, last: T.last, finished: T.finished }) : ''); } catch (_) { /* ignore */ } }
  function loadTimer() {
    try {
      const raw = localStorage.getItem('ordo.timer'); if (!raw) return;
      const s = JSON.parse(raw); Object.assign(T, s);
      if (T.running) { T.remaining -= (Date.now() - T.last) / 1000; T.last = Date.now(); if (T.remaining <= 0) { T.remaining = 0; T.running = false; T.finished = true; } }
      if (T.taskId) startTicking();
    } catch (_) { /* ignore */ }
  }
  function startFocus(taskId) {
    const t = taskById(taskId); if (!t) return;
    if (T.taskId && !T.finished && T.taskId !== taskId && !confirm('Stop the current focus session and start a new one?')) return;
    if (T.taskId && T.taskId !== taskId) stopFocus(true);
    T.taskId = taskId; T.total = settings().pomodoro_minutes * 60; T.remaining = T.total; T.running = true; T.last = Date.now(); T.finished = false;
    startTicking(); saveTimer();
    if ('Notification' in window && Notification.permission === 'default') Notification.requestPermission().catch(() => {});
    toast(`Focus session started · ${settings().pomodoro_minutes} min`);
  }
  function startTicking() {
    if (T.tick) clearInterval(T.tick);
    T.tick = setInterval(() => {
      if (T.running) {
        const now = Date.now(); T.remaining -= (now - T.last) / 1000; T.last = now;
        if (T.remaining <= 0) { T.remaining = 0; finishFocus(); }
        if (Math.floor(T.remaining) % 5 === 0) saveTimer();
      }
      renderFocusWidget();
    }, 1000);
    renderFocusWidget();
  }
  async function finishFocus() {
    T.running = false; T.finished = true; saveTimer();
    beep();
    const t = taskById(T.taskId);
    const minutes = Math.max(1, Math.round(T.total / 60));
    if (t) await run(async () => { await api('POST', `/api/tasks/${T.taskId}/time`, { minutes }); await refresh(); });
    T.logged = true;
    toast(`Focus session complete · ${minutes} min logged${t ? ` to “${t.title.length > 30 ? `${t.title.slice(0, 30)}…` : t.title}”` : ''}`);
    if ('Notification' in window && Notification.permission === 'granted') { try { new Notification('Focus session complete', { body: t ? t.title : 'Take a break.' }); } catch (_) { /* ignore */ } }
    renderFocusWidget();
  }
  async function stopFocus(silent) {
    const elapsed = Math.round((T.total - T.remaining) / 60);
    const id = T.taskId;
    const shouldLog = !T.finished && elapsed >= 1;
    clearInterval(T.tick); T.tick = null;
    Object.assign(T, { taskId: null, total: 0, remaining: 0, running: false, finished: false, logged: false });
    saveTimer(); renderFocusWidget();
    if (shouldLog && id) await run(async () => { await api('POST', `/api/tasks/${id}/time`, { minutes: elapsed }); await refresh(); if (!silent) toast(`${elapsed} min logged`); });
  }
  function beep() {
    try {
      const ctx = new (window.AudioContext || window.webkitAudioContext)();
      [0, 0.18, 0.36].forEach((offset) => { const o = ctx.createOscillator(); const g = ctx.createGain(); o.frequency.value = 880; o.connect(g); g.connect(ctx.destination); g.gain.setValueAtTime(0.0001, ctx.currentTime + offset); g.gain.exponentialRampToValueAtTime(0.2, ctx.currentTime + offset + 0.02); g.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + offset + 0.15); o.start(ctx.currentTime + offset); o.stop(ctx.currentTime + offset + 0.16); });
    } catch (_) { /* audio unavailable */ }
  }
  function renderFocusWidget() {
    const el = $('#focus-widget');
    if (!T.taskId) { el.hidden = true; document.title = 'Ordo — priority-first tasks'; return; }
    const t = taskById(T.taskId);
    const title = t ? t.title : 'Focus';
    const mm = Math.floor(Math.max(0, T.remaining) / 60), ss = Math.floor(Math.max(0, T.remaining) % 60);
    const clock = `${pad(mm)}:${pad(ss)}`;
    const len = 2 * Math.PI * 31;
    el.hidden = false;
    el.classList.toggle('paused', !T.running && !T.finished);
    document.title = T.finished ? 'Done! — Ordo' : `${clock} · ${title}`;
    el.innerHTML = `<div class="fw-head"><span class="pulse"></span>${T.finished ? 'Session complete' : T.running ? 'Focusing' : 'Paused'}<button class="icon-btn icon-btn-xs fw-close" data-focus="discard" title="Close">${icon('x')}</button></div>
      <div class="fw-task" title="${esc(title)}"><a href="#" data-focus="open">${esc(title)}</a></div>
      <div class="fw-body"><div class="fw-ring"><svg viewBox="0 0 74 74"><circle class="track" cx="37" cy="37" r="31"/><circle class="fill" cx="37" cy="37" r="31" stroke-dasharray="${len}" stroke-dashoffset="${len * (1 - (T.total ? (T.total - T.remaining) / T.total : 0))}"/></svg><div class="fw-time">${T.finished ? icon('check') : clock}</div></div>
      <div class="fw-actions">${T.finished
        ? `<button class="btn btn-sm btn-primary" data-focus="complete">${icon('check')}Mark done</button><button class="btn btn-sm" data-focus="again">${icon('play')}Go again</button><button class="btn btn-sm" data-focus="discard" style="flex-basis:100%">Close</button>`
        : `<button class="btn btn-sm ${T.running ? '' : 'btn-primary'}" data-focus="toggle">${T.running ? `${icon('pause')}Pause` : `${icon('play')}Resume`}</button><button class="btn btn-sm" data-focus="extend">+5 min</button><button class="btn btn-sm" data-focus="stop">${icon('square')}Stop & log</button><button class="btn btn-sm" data-focus="complete">${icon('check')}Done</button>`}</div></div>`;
  }
  $('#focus-widget').addEventListener('click', async (e) => {
    const b = e.target.closest('[data-focus]'); if (!b) return;
    e.preventDefault();
    switch (b.dataset.focus) {
      case 'toggle': T.running = !T.running; T.last = Date.now(); saveTimer(); renderFocusWidget(); break;
      case 'extend': T.remaining += 300; T.total += 300; saveTimer(); renderFocusWidget(); break;
      case 'stop': await stopFocus(false); break;
      case 'discard': { const id = T.taskId; clearInterval(T.tick); T.tick = null; Object.assign(T, { taskId: null, total: 0, remaining: 0, running: false, finished: false }); saveTimer(); renderFocusWidget(); if (id && !T.finished) { /* nothing logged */ } break; }
      case 'again': { const id = T.taskId; Object.assign(T, { taskId: null }); startFocus(id); break; }
      case 'complete': { const id = T.taskId; const wasFinished = T.finished; if (wasFinished) { clearInterval(T.tick); T.tick = null; Object.assign(T, { taskId: null, total: 0, remaining: 0, running: false, finished: false }); saveTimer(); renderFocusWidget(); } else { await stopFocus(true); } await toggleComplete(id, null); break; }
      case 'open': select(T.taskId); break;
      default: break;
    }
  });

  // ------------------------------------------------------------------
  // Drag and drop (matrix)
  // ------------------------------------------------------------------
  function bindDragDrop() {
    const content = $('#content');
    content.addEventListener('dragstart', (e) => {
      const row = e.target.closest('.task[draggable="true"]'); if (!row) return;
      e.dataTransfer.setData('text/plain', row.dataset.id); e.dataTransfer.effectAllowed = 'move'; row.classList.add('dragging');
    });
    content.addEventListener('dragend', (e) => { const row = e.target.closest('.task'); if (row) row.classList.remove('dragging'); });
    content.addEventListener('dragover', (e) => { const q = e.target.closest('.quadrant'); if (!q) return; e.preventDefault(); e.dataTransfer.dropEffect = 'move'; q.classList.add('drag-over'); });
    content.addEventListener('dragleave', (e) => { const q = e.target.closest('.quadrant'); if (q && !q.contains(e.relatedTarget)) q.classList.remove('drag-over'); });
    content.addEventListener('drop', async (e) => {
      const q = e.target.closest('.quadrant'); if (!q) return;
      e.preventDefault(); q.classList.remove('drag-over');
      const id = e.dataTransfer.getData('text/plain'); const t = taskById(id);
      if (!t || t.quadrant === q.dataset.quadrant) return;
      await run(async () => {
        const r = await api('POST', `/api/tasks/${id}/quadrant`, { quadrant: q.dataset.quadrant });
        await refresh();
        toast(r.note || `Moved to “${quadrantLabel(q.dataset.quadrant)}”: ${PRIORITIES.find((p) => p.id === r.task.priority).label} priority${r.task.scheduled && r.task.scheduled <= today() ? ', planned for today' : ''}`);
      });
    });
  }

  // ------------------------------------------------------------------
  // Global event wiring
  // ------------------------------------------------------------------
  function onDelegatedClick(e) {
    const actionEl = e.target.closest('[data-action]');
    if (actionEl) { e.preventDefault(); handleAction(actionEl.dataset.action, actionEl, e); return; }
    const row = e.target.closest('.task');
    if (row) {
      if (e.shiftKey || e.ctrlKey || e.metaKey) { toggleSelect(row.dataset.id); }
      else if (S.selection.size) { toggleSelect(row.dataset.id); }
      else select(row.dataset.id);
    }
  }

  function bindEvents() {
    document.addEventListener('click', (e) => {
      if (e.target.closest('#content, #sidebar, #detail, .topbar, #quick-add-wrap')) onDelegatedClick(e);
    });
    $('#content').addEventListener('change', (e) => { if (e.target.dataset.action === 'set-sort') { S.sort = e.target.value; localStorage.setItem('ordo.sort', S.sort); renderContent(); } });
    $('#content').addEventListener('dblclick', (e) => {
      const titleEl = e.target.closest('.task-title'); if (!titleEl || titleEl.querySelector('.inline-edit')) return;
      const row = titleEl.closest('.task'); const t = row && taskById(row.dataset.id);
      if (!t || t.deleted_at) return;
      e.preventDefault();
      window.getSelection()?.removeAllRanges();
      startInlineEdit(titleEl, t);
    });

    // Detail panel field edits.
    const detail = $('#detail');
    detail.addEventListener('change', async (e) => {
      const f = e.target.dataset.field; if (!f) return;
      const id = S.selectedId; const t = taskById(id); if (!t) return;
      const v = e.target.value;
      switch (f) {
        case 'title': if (v.trim() && v.trim() !== t.title) patchTask(id, { title: v.trim() }); else e.target.value = t.title; break;
        case 'notes': patchTask(id, { notes: v }); break;
        case 'project_id': patchTask(id, { project_id: v || null }); break;
        case 'due_date': patchTask(id, { due_date: v || null }); break;
        case 'due_time': patchTask(id, { due_time: v ? `${v}:00` : null }); break;
        case 'scheduled': patchTask(id, { scheduled: v || null }); break;
        case 'estimate_minutes': patchTask(id, { estimate_minutes: v ? Number(v) : null }); break;
        case 'recurrence': patchTask(id, { recurrence: recurrenceFromKey(v, t) }); break;
        case 'sub-title': { const sid = e.target.closest('.sub-row').dataset.sid; if (v.trim()) await run(async () => { await api('PATCH', `/api/tasks/${id}/subtasks/${sid}`, { title: v.trim() }); await refresh(); }); break; }
        default: break;
      }
    });
    detail.addEventListener('input', (e) => { if (e.target.classList.contains('detail-title')) autosize(e.target); });
    detail.addEventListener('keydown', async (e) => {
      const id = S.selectedId;
      if (e.target.classList.contains('detail-title') && e.key === 'Enter') { e.preventDefault(); e.target.blur(); }
      if (e.target.id === 'sub-input' && e.key === 'Enter') {
        const title = e.target.value.trim(); if (!title) return;
        e.preventDefault(); S.focusAfter = '#sub-input';
        await run(async () => { await api('POST', `/api/tasks/${id}/subtasks`, { title }); await refresh(); });
      }
      if (e.target.id === 'tag-input' && (e.key === 'Enter' || e.key === ',')) {
        e.preventDefault();
        const tag = e.target.value.replace(/[,#@]/g, '').trim(); if (!tag) return;
        const t = taskById(id); S.focusAfter = '#tag-input';
        await patchTask(id, { tags: [...t.tags, tag] });
      }
      if (e.target.id === 'tag-input' && e.key === 'Backspace' && !e.target.value) {
        const t = taskById(id); if (t.tags.length) { S.focusAfter = '#tag-input'; await patchTask(id, { tags: t.tags.slice(0, -1) }); }
      }
    });

    // Quick add.
    $('#quick-add-form').addEventListener('submit', (e) => { e.preventDefault(); submitQuickAdd(); });
    $('#quick-add').addEventListener('input', () => { clearTimeout(previewTimer); previewTimer = setTimeout(updatePreview, 120); });
    $('#quick-add').addEventListener('keydown', (e) => { if (e.key === 'Escape') { e.target.value = ''; $('#parse-preview').hidden = true; e.target.blur(); } });

    // Search.
    $('#search-input').addEventListener('input', (e) => { clearTimeout(searchTimer); searchTimer = setTimeout(() => doSearch(e.target.value), 160); });
    $('#search-input').addEventListener('keydown', (e) => { if (e.key === 'Escape') { e.target.value = ''; doSearch(''); e.target.blur(); } });

    // Bulk bar.
    $('#bulk-bar').addEventListener('click', (e) => {
      const b = e.target.closest('[data-bulk]'); if (!b) return;
      const action = b.dataset.bulk;
      if (action === 'purge') { confirmDialog({ title: 'Delete forever?', message: `${S.selection.size} tasks will be permanently deleted.`, confirmLabel: 'Delete forever', danger: true }).then((ok) => { if (ok) bulk('purge'); }); return; }
      bulk(action, b.dataset.value);
    });
    $('#bulk-bar').addEventListener('change', (e) => {
      const kind = e.target.dataset.bulkSelect; if (!kind || !e.target.value) return;
      const v = e.target.value;
      bulk(kind, kind === 'project' && v === '__inbox' ? null : v);
    });

    // Modal / palette backdrops.
    $('#modal').addEventListener('click', (e) => {
      if (e.target === e.currentTarget || e.target.closest('[data-modal="close"]')) { e.currentTarget.dispatchEvent(new Event('modal-closed')); closeModal(); }
    });
    $('#palette').addEventListener('click', (e) => {
      const item = e.target.closest('.palette-item');
      if (item) { const it = paletteItems(S.paletteQuery)[Number(item.dataset.index)]; closePalette(); if (it) it.run(); return; }
      if (e.target === e.currentTarget) closePalette();
    });

    // Chart tooltips.
    let tip = null;
    document.addEventListener('mouseover', (e) => {
      const el = e.target.closest('[data-tip]'); if (!el) return;
      tip = document.createElement('div'); tip.className = 'chart-tip'; tip.textContent = el.dataset.tip; document.body.appendChild(tip);
      const move = (ev) => { tip.style.left = `${ev.clientX + 12}px`; tip.style.top = `${ev.clientY - 30}px`; };
      move(e);
      el.addEventListener('mousemove', move);
      el.addEventListener('mouseleave', () => { if (tip) tip.remove(); tip = null; el.removeEventListener('mousemove', move); }, { once: true });
    });

    // Keyboard shortcuts.
    document.addEventListener('keydown', (e) => {
      const typing = ['INPUT', 'TEXTAREA', 'SELECT'].includes(e.target.tagName) || e.target.isContentEditable;
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); if (S.paletteOpen) closePalette(); else openPalette(); return; }
      if (e.key === 'Escape') {
        if (S.paletteOpen) { closePalette(); return; }
        if (modalOpen()) { $('#modal').dispatchEvent(new Event('modal-closed')); closeModal(); return; }
        if (typing) { e.target.blur(); return; }
        if (S.selection.size) { S.selection.clear(); render(); return; }
        if (S.selectedId) { closeDetail(); return; }
        if (S.search) { $('#search-input').value = ''; doSearch(''); }
        return;
      }
      if (typing || S.paletteOpen || modalOpen() || e.ctrlKey || e.metaKey || e.altKey) return;
      const k = e.key;
      const viewByKey = VIEWS.find((v) => v.key === k);
      if (viewByKey) { go({ type: viewByKey.id, id: null }); return; }
      switch (k) {
        case 'n': case 'N': e.preventDefault(); focusQuickAdd(); break;
        case '/': e.preventDefault(); $('#search-input').focus(); break;
        case '?': openShortcuts(); break;
        case 'd': case 'D': toggleTheme(); break;
        case 'j': case 'k': case 'ArrowDown': case 'ArrowUp': {
          const rows = $$('#content .task'); if (!rows.length) return;
          e.preventDefault();
          const idx = rows.findIndex((r) => r.dataset.id === S.selectedId);
          const next = (k === 'j' || k === 'ArrowDown') ? Math.min(rows.length - 1, idx + 1) : Math.max(0, idx - 1);
          select(rows[next].dataset.id);
          const el = $(`#content .task[data-id="${rows[next].dataset.id}"]`); if (el) el.scrollIntoView({ block: 'nearest' });
          break;
        }
        case 'c': case 'C': if (S.selectedId) toggleComplete(S.selectedId, $(`#content .task[data-id="${S.selectedId}"]`)); break;
        case 's': case 'S': if (S.selectedId) { const t = taskById(S.selectedId); patchTask(S.selectedId, { starred: !t.starred }); } break;
        case 't': case 'T': if (S.selectedId) patchTask(S.selectedId, { scheduled: today() }); break;
        case 'f': case 'F': if (S.selectedId) startFocus(S.selectedId); break;
        case 'Delete': case 'Backspace': if (S.selectedId) trashTask(S.selectedId); break;
        default: break;
      }
    });
  }

  // ------------------------------------------------------------------
  // Boot
  // ------------------------------------------------------------------
  async function boot() {
    applyTheme();
    if (localStorage.getItem('ordo.sidebar') === 'collapsed') $('#app').classList.add('sidebar-collapsed');
    bindEvents();
    bindDragDrop();
    S.view = hashToView();
    try {
      S.data = await api('GET', '/api/bootstrap');
      if (S.view.type === 'analytics') S.analytics = await api('GET', '/api/analytics');
    } catch (e) {
      $('#content').innerHTML = `<div class="card"><div class="empty"><div class="empty-icon">${icon('alert')}</div><h3>Could not reach the Ordo server</h3><p>${esc(e.message)}. Is <code>cargo run</code> still running?</p></div></div>`;
      return;
    }
    render();
    loadTimer();
  }

  boot();
})();
