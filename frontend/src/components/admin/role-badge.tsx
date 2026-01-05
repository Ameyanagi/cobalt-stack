import { Shield, User } from 'lucide-react'
import { Badge } from '@/components/ui/badge'

interface RoleBadgeProps {
  role: 'admin' | 'user'
}

export function RoleBadge({ role }: RoleBadgeProps) {
  if (role === 'admin') {
    return (
      <Badge variant="default" className="gap-1">
        <Shield className="h-3 w-3" />
        Admin
      </Badge>
    )
  }

  return (
    <Badge variant="secondary" className="gap-1">
      <User className="h-3 w-3" />
      User
    </Badge>
  )
}
