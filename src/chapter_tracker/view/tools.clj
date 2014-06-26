(ns chapter-tracker.view.tools (:gen-class))

(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(java.awt.event ActionListener)
  '(javax.swing JFrame JPanel JButton JFileChooser JLabel JTextField JOptionPane)
  '(javax.swing.table DefaultTableModel TableCellRenderer TableCellEditor)
)

(defmacro create-frame [properties & body]
  `(let [~'container (JFrame.)
         ~'frame ~'container
         layout# (GridBagLayout.)]
     ~(if (contains? properties :title) `(.setTitle ~'container ~(:title properties)))
     (.setLayout ~'container layout#)
     ~(if (or (contains? properties :width) (contains? properties :height))
        `(.setPreferredSize ~'container (Dimension.
                                          ~(if (contains? properties :width) (:width properties) `(.getWidth ~'container))
                                          ~(if (contains? properties :height) (:height properties) `(.getHeight ~'container))
                                        ))
      )
     ~@body
     (.layoutContainer layout# ~'container)
     (.pack ~'frame)
     ~'container
   )
)

(defmacro create-panel [properties & body]
  `(let [~'container (JPanel.)
         ~'panel ~'container
         layout# (GridBagLayout.)]
     (.setLayout ~'container layout#)
     (.setBorder ~'panel (javax.swing.BorderFactory/createLineBorder java.awt.Color/LIGHT_GRAY))
     ~(if (or (contains? properties :width) (contains? properties :height))
        `(.setPreferredSize ~'container (Dimension.
                                          ~(if (contains? properties :width) (:width properties) `(.getWidth ~'container))
                                          ~(if (contains? properties :height) (:height properties) `(.getHeight ~'container))
                                        ))
      )
     ~@body
     (.layoutContainer layout# ~'container)
     ~'container
   )
)

(defmacro add-with-constraints [component & constraints]
  (let [grid-bag-constraints (gensym)]
    `(.add ~'container ~component (let [~grid-bag-constraints (GridBagConstraints.)]
                                    (set! (. ~grid-bag-constraints anchor) GridBagConstraints/NORTHWEST)
                                    ~@(map (fn [[method arg]]
                                             `(set! (. ~grid-bag-constraints ~method) ~arg)
                                           ) constraints)
                                    ~grid-bag-constraints
                                  )
     )
  )
)

(defmacro action-button [caption & body]
  `(let [result# (JButton. ~caption)]
     (.addActionListener result# (proxy [ActionListener] []
                                   (actionPerformed [e#]
                                     ~@body
                                   )
                                 ))
     result#
   )
)

(defn choose-*
  ([target-type] (choose-* target-type "."))
  ([target-type start-from]
   (let [chooser (JFileChooser. start-from)]
     (.setFileSelectionMode chooser target-type)
     (condp = (.showOpenDialog chooser nil)
       JFileChooser/APPROVE_OPTION (.. chooser getSelectedFile toString)
       JFileChooser/CANCEL_OPTION nil
     )
   ))
)

(defn choose-dir
  ([] (choose-* JFileChooser/DIRECTORIES_ONLY))
  ([start-from] (choose-* JFileChooser/DIRECTORIES_ONLY start-from))
)
(defn choose-file
  ([] (choose-* JFileChooser/FILES_ONLY))
  ([start-from] (choose-* JFileChooser/FILES_ONLY start-from))
)

(defn create-delete-dialog[item-type item-name on-confirmation-function]
  (create-frame {:title (str "Delete " item-type)}
                (let [
                      confirmation-textbox (JTextField. 20)
                     ]
                  (add-with-constraints (JLabel. (str "This will delete the " item-type)) (gridx 0) (gridy 0))
                  (add-with-constraints (JLabel. item-name) (gridx 0) (gridy 1))
                  (add-with-constraints (JLabel. "Type \"yes\" in the textbox to configrm") (gridx 0) (gridy 2))
                  (add-with-constraints confirmation-textbox (gridx 0) (gridy 3))
                  (add-with-constraints (action-button "CONFIRM"
                                                       (if (= "yes" (.getText confirmation-textbox))
                                                         (do
                                                           (on-confirmation-function)
                                                           (.dispose frame))
                                                         (JOptionPane/showMessageDialog
                                                           nil
                                                           (str "Type \"yes\" if you really want to delete\n" item-name)
                                                           "Error - unable to delete" JOptionPane/ERROR_MESSAGE)
                                                       )
                                              )
                                        (gridx 0) (gridy 4) (fill GridBagConstraints/HORIZONTAL))
                )
  )
)
