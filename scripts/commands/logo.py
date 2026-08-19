"""
Logo generation and vector/raster export command for GoldSrc.rs.
Generates clean, scalable SVG and PNG logos in various styles:
  - flat: Clean minimalist vector (classic flat orange)
  - 3d: Volumetric 3D beveled with heavy industrial rust patina and steel pitting
  - rust: Weathered dark cast-iron steel and oxidized rust patina
"""

import argparse
import math
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional


def generate_gear_path(
    cx: float,
    cy: float,
    outer_r: float,
    root_r: float,
    inner_r: float,
    teeth_count: int = 14,
    top_width_fraction: float = 0.38,
    root_width_fraction: float = 0.62,
    offset_x: float = 0.0,
    offset_y: float = 0.0,
) -> str:
    """Generate SVG path definition for the mechanical gear."""
    angle_per_tooth = 2 * math.pi / teeth_count
    gear_pts = []

    for i in range(teeth_count):
        mid_angle = -math.pi / 2 + i * angle_per_tooth
        half_top = (angle_per_tooth * top_width_fraction) / 2
        half_root = (angle_per_tooth * root_width_fraction) / 2

        a_root_left = mid_angle - half_root
        a_top_left = mid_angle - half_top
        a_top_right = mid_angle + half_top
        a_root_right = mid_angle + half_root

        p1 = (
            cx + offset_x + root_r * math.cos(a_root_left),
            cy + offset_y + root_r * math.sin(a_root_left),
        )
        p2 = (
            cx + offset_x + outer_r * math.cos(a_top_left),
            cy + offset_y + outer_r * math.sin(a_top_left),
        )
        p3 = (
            cx + offset_x + outer_r * math.cos(a_top_right),
            cy + offset_y + outer_r * math.sin(a_top_right),
        )
        p4 = (
            cx + offset_x + root_r * math.cos(a_root_right),
            cy + offset_y + root_r * math.sin(a_root_right),
        )

        gear_pts.extend([p1, p2, p3, p4])

    d_gear = f"M {gear_pts[0][0]:.2f} {gear_pts[0][1]:.2f} " + " ".join(
        f"L {p[0]:.2f} {p[1]:.2f}" for p in gear_pts[1:]
    ) + " Z"

    d_hole = (
        f" M {cx + offset_x - inner_r:.2f} {cy + offset_y:.2f}"
        f" A {inner_r:.2f} {inner_r:.2f} 0 1 0 {cx + offset_x + inner_r:.2f} {cy + offset_y:.2f}"
        f" A {inner_r:.2f} {inner_r:.2f} 0 1 0 {cx + offset_x - inner_r:.2f} {cy + offset_y:.2f} Z"
    )

    return d_gear + d_hole


def generate_lambda_path(
    cx: float, cy: float, scale: float, offset_x: float = 0.0, offset_y: float = 0.0
) -> str:
    """Generate SVG path definition for the authentic Half-Life Lambda glyph."""
    stem_angle = math.radians(60.0)
    cos_s, sin_s = math.cos(stem_angle), math.sin(stem_angle)
    ux, uy = cos_s, sin_s
    nx, ny = sin_s, -cos_s

    thickness = 17.5
    foot_len = 11.0
    foot_height = 16.0

    top_y = -62.0
    bot_y = 52.0
    top_right_x = 8.0
    cap_left_x = -28.0
    cap_bot_y = -46.0
    fork_y = -30.0

    p1 = (cap_left_x, top_y)
    p2 = (top_right_x, top_y)

    p_inner_top = (p2[0] - thickness * nx, p2[1] - thickness * ny)

    t_base = (bot_y - p_inner_top[1]) / uy
    p6 = (p_inner_top[0] + t_base * ux, bot_y)

    bend_angle = math.radians(78.0)
    foot_angle = stem_angle - bend_angle
    wx, wy = math.cos(foot_angle), math.sin(foot_angle)

    det_w = uy * wx - ux * wy
    s_foot = foot_len + thickness / det_w

    p5 = (p6[0] + s_foot * wx, p6[1] + s_foot * wy)
    p4 = (p5[0] - foot_height * ux, p5[1] - foot_height * uy)
    p3 = (p4[0] - foot_len * wx, p4[1] - foot_len * wy)

    def pt_on_stem_inner(target_y: float):
        t = (target_y - p6[1]) / (-uy)
        return (p6[0] - t * ux, target_y)

    p11 = pt_on_stem_inner(cap_bot_y)
    p10 = pt_on_stem_inner(fork_y)

    left_angle = math.radians(59.0)
    cos_l, sin_l = math.cos(left_angle), math.sin(left_angle)
    vx, vy = -cos_l, sin_l

    t_leg = (bot_y - fork_y) / vy
    p9 = (p10[0] + t_leg * vx, bot_y)
    p8 = (p9[0] + thickness / sin_l, bot_y)

    dx_crotch = p8[0] - p6[0]
    dy_crotch = p8[1] - p6[1]
    det_crotch = (-ux) * (-vy) - (-uy) * (-vx)
    s_crotch = (dx_crotch * (-vy) - dy_crotch * (-vx)) / det_crotch
    p7 = (p6[0] - s_crotch * ux, p6[1] - s_crotch * uy)

    p12 = (cap_left_x, cap_bot_y)

    raw_points = [p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11, p12]

    pts = [(cx + offset_x + x * scale, cy + offset_y + y * scale) for x, y in raw_points]
    d_lambda = f"M {pts[0][0]:.2f} {pts[0][1]:.2f} " + " ".join(
        f"L {p[0]:.2f} {p[1]:.2f}" for p in pts[1:]
    ) + " Z"
    return d_lambda


def render_svg_content(
    size: int = 512,
    color: str = "#E54C1A",
    style: str = "flat",
) -> str:
    """Render raw SVG string for the specified style."""
    cx, cy = size / 2, size / 2

    scale = size * 0.0033
    lambda_cx = cx - size * 0.045
    lambda_cy = cy + size * 0.006

    leg_thickness = 17.5 * scale

    outer_r = size * 0.45
    tooth_height = size * 0.075
    root_r = outer_r - tooth_height
    inner_r = root_r - leg_thickness

    gear_path = generate_gear_path(cx, cy, outer_r, root_r, inner_r, teeth_count=14)
    lambda_path = generate_lambda_path(lambda_cx, lambda_cy, scale)

    if style == "flat":
        return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}">
  <!-- GoldSrc.rs Vector Logo (Flat Orange) -->
  <path fill="{color}" fill-rule="evenodd" d="{gear_path}" />
  <path fill="{color}" d="{lambda_path}" />
</svg>
'''
    elif style == "3d":
        extrusion_depth = 6
        ext_layers = []
        for d in range(extrusion_depth, 0, -1):
            offset = d * 1.0
            g_p = generate_gear_path(
                cx, cy, outer_r, root_r, inner_r, teeth_count=14, offset_x=0.0, offset_y=offset
            )
            l_p = generate_lambda_path(lambda_cx, lambda_cy, scale, offset_x=0.0, offset_y=offset)
            ext_layers.append(
                f'    <path fill="#3B1202" fill-rule="evenodd" d="{g_p}" />\n'
                f'    <path fill="#3B1202" d="{l_p}" />'
            )
        extrusion_svg = "\n".join(ext_layers)

        return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}">
  <defs>
    <!-- Heavy Ambient Drop Shadow -->
    <filter id="ambientShadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="7" stdDeviation="6" flood-color="#000000" flood-opacity="0.55" />
    </filter>

    <!-- Rich, Saturated Half-Life Orange & Industrial Steel Base -->
    <radialGradient id="vibrantRustBase" cx="45%" cy="38%" r="65%">
      <stop offset="0%" stop-color="#F26214" />
      <stop offset="30%" stop-color="#E24D05" />
      <stop offset="60%" stop-color="#C53A02" />
      <stop offset="85%" stop-color="#902300" />
      <stop offset="100%" stop-color="#521300" />
    </radialGradient>

    <!-- Subtle Edge Chamfer Light -->
    <linearGradient id="warmEdgeBevel" x1="20%" y1="10%" x2="80%" y2="90%">
      <stop offset="0%" stop-color="#F59E42" stop-opacity="0.25" />
      <stop offset="35%" stop-color="#D9530A" stop-opacity="0.08" />
      <stop offset="70%" stop-color="#1A0701" stop-opacity="0.4" />
      <stop offset="100%" stop-color="#0A0200" stop-opacity="0.75" />
    </linearGradient>

    <!-- High-Contrast 3D Rust & Exposed Steel Metal Shader -->
    <filter id="realRustShader" x="-10%" y="-10%" width="120%" height="120%">
      <!-- Macro patches for rust & raw steel variation -->
      <feTurbulence type="turbulence" baseFrequency="0.022" numOctaves="5" seed="88" result="macroNoise" />
      <!-- Micro iron grain & steel scratches/pits -->
      <feTurbulence type="fractalNoise" baseFrequency="0.08" numOctaves="4" seed="42" result="microNoise" />

      <!-- Combined heightmap for bumps -->
      <feComposite in="macroNoise" in2="microNoise" operator="arithmetic" k1="0" k2="0.5" k3="0.35" result="bumpMap" />

      <!-- 3D Surface Relief Lighting -->
      <feDiffuseLighting in="bumpMap" surfaceScale="2.0" diffuseConstant="0.95" lighting-color="#F2DEC6" result="bumpLight">
        <feDistantLight azimuth="225" elevation="55" />
      </feDiffuseLighting>

      <!-- Color Map: Rich Orange Rust with Exposed Steel Gray/Metallic patches -->
      <feColorMatrix in="macroNoise" type="matrix" values="
        1.30  0.28  0.00  0  0.15
        0.50  0.22  0.00  0  0.03
        0.15  0.10  0.25  0  0.05
        0.00  0.00  0.00  1  0" result="rustAndSteelMap" />

      <!-- Clip colors to source -->
      <feComposite in="rustAndSteelMap" in2="SourceGraphic" operator="in" result="clippedTexture" />

      <!-- Overlay texture onto vibrant orange base -->
      <feBlend mode="overlay" in="SourceGraphic" in2="clippedTexture" result="texturedBase" />

      <!-- Soft multiply with 3D relief lighting -->
      <feComposite in="texturedBase" in2="bumpLight" operator="arithmetic" k1="0.68" k2="0.36" k3="0.0" k4="0" result="litRustedMetal" />

      <!-- Perimeter Chamfer Bevel Highlight -->
      <feGaussianBlur in="SourceAlpha" stdDeviation="1.2" result="edgeBlur" />
      <feSpecularLighting in="edgeBlur" surfaceScale="1.6" specularConstant="0.45" specularExponent="24" lighting-color="#F5CE98" result="edgeSpecular">
        <feDistantLight azimuth="225" elevation="55" />
      </feSpecularLighting>
      <feComposite in="edgeSpecular" in2="SourceAlpha" operator="in" result="clippedEdgeSpecular" />

      <!-- Composite lit metal with soft edge specular -->
      <feComposite in="litRustedMetal" in2="clippedEdgeSpecular" operator="arithmetic" k1="0" k2="1" k3="0.3" k4="0" result="unclippedResult" />

      <!-- Strict Alpha Masking -->
      <feComposite in="unclippedResult" in2="SourceAlpha" operator="in" />
    </filter>
  </defs>

  <!-- 1. Ambient Drop Shadow & Dark Oxidized 3D Extrusion Walls -->
  <g filter="url(#ambientShadow)">
{extrusion_svg}
  </g>

  <!-- 2. Vibrant 3D Corroded Rusted Metal & Exposed Steel Surface -->
  <g filter="url(#realRustShader)">
    <path fill="url(#vibrantRustBase)" fill-rule="evenodd" d="{gear_path}" />
    <path fill="url(#vibrantRustBase)" d="{lambda_path}" />
  </g>

  <!-- 3. Soft Warm Chamfer Edge Highlight Overlay -->
  <path fill="url(#warmEdgeBevel)" fill-rule="evenodd" d="{gear_path}" mix-blend-mode="overlay" opacity="0.45" />
  <path fill="url(#warmEdgeBevel)" d="{lambda_path}" mix-blend-mode="overlay" opacity="0.45" />
</svg>
'''
    elif style == "rust":
        return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}">
  <defs>
    <linearGradient id="metalGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#4B443D" />
      <stop offset="25%" stop-color="#2D2926" />
      <stop offset="50%" stop-color="#5A524B" />
      <stop offset="75%" stop-color="#34302C" />
      <stop offset="100%" stop-color="#1E1B19" />
    </linearGradient>

    <radialGradient id="rustPatina" cx="45%" cy="40%" r="60%">
      <stop offset="0%" stop-color="#E85D1A" stop-opacity="0.95" />
      <stop offset="35%" stop-color="#C44812" stop-opacity="0.85" />
      <stop offset="70%" stop-color="#8B310B" stop-opacity="0.9" />
      <stop offset="100%" stop-color="#4A1804" stop-opacity="0.95" />
    </radialGradient>

    <linearGradient id="bevelLight" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#FFE2C2" stop-opacity="0.4" />
      <stop offset="50%" stop-color="#000000" stop-opacity="0" />
      <stop offset="100%" stop-color="#000000" stop-opacity="0.6" />
    </linearGradient>

    <filter id="rustTexture" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.045" numOctaves="4" result="noise" />
      <feColorMatrix type="matrix" values="
        1.2  0.2  0.0  0  0.25
        0.5  0.3  0.0  0  0.08
        0.1  0.0  0.0  0  0.02
        0.0  0.0  0.0  1  0" in="noise" result="rustColor" />
      <feComposite in2="SourceGraphic" in="rustColor" operator="in" result="texturedRust" />
      <feBlend mode="multiply" in="SourceGraphic" in2="texturedRust" result="blended" />
      <feComposite in="blended" in2="SourceAlpha" operator="in" />
    </filter>

    <filter id="metalShadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="3" dy="5" stdDeviation="4" flood-color="#000000" flood-opacity="0.65" />
    </filter>
  </defs>

  <g filter="url(#metalShadow)">
    <path fill="url(#metalGrad)" fill-rule="evenodd" d="{gear_path}" />
    <path fill="url(#metalGrad)" d="{lambda_path}" />

    <g filter="url(#rustTexture)">
      <path fill="url(#rustPatina)" fill-rule="evenodd" d="{gear_path}" />
      <path fill="url(#rustPatina)" d="{lambda_path}" />
    </g>

    <path fill="url(#bevelLight)" fill-rule="evenodd" d="{gear_path}" opacity="0.7" />
    <path fill="url(#bevelLight)" d="{lambda_path}" opacity="0.7" />
  </g>
</svg>
'''
    raise ValueError(f"Unknown style: {style}")


def convert_svg_to_png(svg_path: Path, png_path: Path, size: int) -> bool:
    """Attempt conversion of SVG to PNG using available tools."""
    # 1. Try cairosvg (only if native cairo library is present)
    try:
        import cairosvg

        cairosvg.svg2png(
            url=str(svg_path.resolve()),
            write_to=str(png_path.resolve()),
            output_width=size,
            output_height=size,
        )
        if png_path.exists():
            return True
    except (ImportError, OSError, Exception):
        pass

    # 2. Try svglib + reportlab
    try:
        from reportlab.graphics import renderPM
        from svglib.svglib import svg2rlg

        drawing = svg2rlg(str(svg_path.resolve()))
        if drawing:
            renderPM.drawToFile(drawing, str(png_path.resolve()), fmt="PNG")
            if png_path.exists():
                return True
    except (ImportError, OSError, Exception):
        pass

    # 3. Try resvg CLI if available (Rust native SVG renderer)
    resvg = shutil.which("resvg")
    if resvg:
        try:
            cmd = [resvg, "-w", str(size), "-h", str(size), str(svg_path.resolve()), str(png_path.resolve())]
            res = subprocess.run(cmd, capture_output=True)
            if res.returncode == 0 and png_path.exists():
                return True
        except Exception:
            pass

    # 4. Try inkscape CLI if available in PATH
    inkscape = shutil.which("inkscape")
    if inkscape:
        try:
            cmd = [
                inkscape,
                str(svg_path.resolve()),
                f"--export-filename={png_path.resolve()}",
                f"--export-width={size}",
                f"--export-height={size}",
            ]
            res = subprocess.run(cmd, capture_output=True)
            if res.returncode == 0 and png_path.exists():
                return True
        except Exception:
            pass

    # 5. Try msedge / chrome headless screenshot
    browser_exe = None
    for name in ["msedge", "chrome", "google-chrome", "chromium"]:
        p = shutil.which(name)
        if p:
            browser_exe = p
            break

    # Standard Windows Edge default paths fallback
    if not browser_exe and sys.platform == "win32":
        for default_edge in [
            Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
            Path(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
        ]:
            if default_edge.exists():
                browser_exe = str(default_edge)
                break

    if browser_exe:
        try:
            cmd = [
                browser_exe,
                "--headless",
                "--disable-gpu",
                "--hide-scrollbars",
                f"--window-size={size},{size}",
                f"--screenshot={png_path.resolve()}",
                f"file:///{svg_path.resolve().as_posix()}",
            ]
            res = subprocess.run(cmd, capture_output=True)
            if png_path.exists():
                return True
        except Exception:
            pass

    return False


def generate_single_logo(
    out_dir: Path,
    base_name: str,
    style: str,
    size: int,
    fmt: str,
    color: str,
) -> None:
    """Generate logo files for a specific style and format."""
    out_dir.mkdir(parents=True, exist_ok=True)
    svg_content = render_svg_content(size=size, color=color, style=style)

    svg_path = out_dir / f"{base_name}.svg"
    png_path = out_dir / f"{base_name}.png"

    # Always write SVG first
    svg_path.write_text(svg_content, encoding="utf-8")
    print(f"  [OK] Generated SVG ({style}): {svg_path}")

    if fmt in ["png", "all"]:
        converted = convert_svg_to_png(svg_path, png_path, size)
        if converted:
            print(f"  [OK] Exported PNG ({size}x{size}): {png_path}")
        else:
            print(
                f"  [Notice] PNG conversion skipped (install 'cairosvg' or 'svglib' for PNG rasterization: `pip install cairosvg`).",
                file=sys.stderr,
            )

    if fmt == "png" and svg_path.exists() and png_path.exists():
        # User only requested PNG and it succeeded
        pass


def main(argv: Optional[list[str]] = None) -> int:
    """CLI entrypoint for logo generation."""
    parser = argparse.ArgumentParser(
        prog="python -m scripts logo",
        description="Generate vector (SVG) and raster (PNG) brand logos for GoldSrc.rs",
    )
    parser.add_argument(
        "--style",
        "-s",
        choices=["flat", "3d", "rust", "all"],
        default="all",
        help="Visual style: 'flat' (minimalist orange), '3d' (beveled rust patina), 'rust' (dark steel), 'all' (all 3)",
    )
    parser.add_argument(
        "--format",
        "-f",
        choices=["svg", "png", "all"],
        default="all",
        help="Output format: 'svg', 'png', or 'all' (default: all)",
    )
    parser.add_argument(
        "--size",
        type=int,
        default=512,
        help="Dimension in pixels (width/height, default: 512)",
    )
    parser.add_argument(
        "--color",
        default="#E54C1A",
        help="Hex color for 'flat' style (default: #E54C1A)",
    )
    parser.add_argument(
        "--out-dir",
        "-d",
        type=Path,
        default=Path("private/assets/logos"),
        help="Output directory (default: private/assets/logos)",
    )
    parser.add_argument(
        "--name",
        "-n",
        default=None,
        help="Custom base filename (without extension). If omitted, defaults to logo_<style>.",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=None,
        help="Explicit output filepath (overrides --out-dir and --name). Extension determines format if specified.",
    )

    args = parser.parse_args(argv)

    print("\n========================================")
    print("       GoldSrc.rs Logo Generator        ")
    print("========================================\n")

    # If explicit output path is given
    if args.output:
        out_file = Path(args.output)
        out_dir = out_file.parent
        base_name = out_file.stem
        ext = out_file.suffix.lower().lstrip(".")
        target_fmt = ext if ext in ["svg", "png"] else args.format
        style = "flat" if args.style == "all" else args.style
        generate_single_logo(
            out_dir=out_dir,
            base_name=base_name,
            style=style,
            size=args.size,
            fmt=target_fmt,
            color=args.color,
        )
        return 0

    styles = ["flat", "3d", "rust"] if args.style == "all" else [args.style]

    for style in styles:
        if args.name:
            base_name = args.name if len(styles) == 1 else f"{args.name}_{style}"
        else:
            base_name = "logo" if style == "flat" else f"logo_{style}"

        generate_single_logo(
            out_dir=args.out_dir,
            base_name=base_name,
            style=style,
            size=args.size,
            fmt=args.format,
            color=args.color,
        )

    print(f"\n[SUCCESS] Logo generation complete in '{args.out_dir}'!\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
