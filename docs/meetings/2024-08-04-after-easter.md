
- Need to update to lastest verona-rt upstream
    - https://github.com/microsoft/verona-rt/pull/37

- It's better to call schedule_lambda on Requests than Cowns.
- schedule_lambda has weird possibly wrong generics.

- TODO:
    - Find requirements for extra bytes
    - Schedule Behaviours
    - Schedule closures.

- For lambdas
    - Overwrite Behaviour::make
    - Can keep BehaviorCore::make