// Measurements and colors mirror src/components/{voice_list,user_cell,notification_list}.rs
// and src/theme.rs. This is illustrative content, not a live connection.
export function OverlayPreview() {
  return (
    <figure className="overlay-example">
      <div className="overlay-viewport">
        <div className="overlay-screen" role="img" aria-label="Example Yaplay overlay: You, Mara speaking, Theo muted, and a notification in the top right.">
          <div className="voice-panel">
            <div className="voice-heading">Voice</div>
            <div className="voice-members">
              <div className="voice-member"><span className="voice-avatar self">Y</span><span>You (you)</span></div>
              <div className="voice-member"><span className="voice-avatar speaking">M</span><span>Mara</span></div>
              <div className="voice-member muted"><span className="voice-avatar theo">T</span><span>Theo</span><span className="mute-dot" /></div>
            </div>
          </div>
          <div className="notification-panel"><div>Call started</div><div className="notification-body">Mara joined the call.</div></div>
        </div>
      </div>
      <figcaption>Example overlay with sample data. The background represents your desktop.</figcaption>
    </figure>
  );
}
