export default function MemberList({ members }) {
  return (
    <div className="member-list">
      {members.map((member) => (
        <div className="member-card" key={member.id}>
          <span className="member-id">#{member.id}</span>
          <span>{member.name}</span>
        </div>
      ))}
    </div>
  )
}
